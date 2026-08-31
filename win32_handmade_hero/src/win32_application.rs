use crate::application_loader::{ApplicationLoader, ApplicationStub};
use crate::direct_sound::DirectSound;
use crate::direct_sound_buffer::DirectSoundBuffer;
use crate::performance_counter::PerformanceCounter;
use crate::playback_recorder::PlaybackRecorder;
use crate::win32_controller::{Win32Controller, Win32ControllerState};
use crate::win32_keyboard::Win32Keyboard;
use crate::win32_mouse::Win32Mouse;
use crate::win32_window::Win32Window;
use core::slice;
use handmade_hero_interface::application::Application;
use handmade_hero_interface::application_error::{ApplicationError, Result};
use handmade_hero_interface::audio_context::AudioContext;
use handmade_hero_interface::back_buffer::BackBuffer;
use handmade_hero_interface::button_state::ButtonState;
use handmade_hero_interface::controller_state::ControllerState;
use handmade_hero_interface::game_state::GameState;
use handmade_hero_interface::initialize_context::InitializeContext;
use handmade_hero_interface::input_context::InputContext;
use handmade_hero_interface::input_state::InputState;
use handmade_hero_interface::render_context::RenderContext;
use handmade_hero_interface::stereo_sample::StereoSample;
use handmade_hero_interface::units::si::frequency::Frequency;
use handmade_hero_interface::units::si::information::Information;
use handmade_hero_interface::units::si::length::pixel;
use std::cmp::Ordering;
use std::ffi::c_void;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;
use uom::num::{Saturating, Zero};
use uom::si::f32::{Ratio, Time};
use uom::si::frequency::hertz;
use uom::si::information::byte;
use uom::si::length::Length;
use uom::si::ratio::ratio;
use uom::si::time::second;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::{DEVMODEW, ENUM_CURRENT_SETTINGS, EnumDisplaySettingsW};
use windows::Win32::Media::{TIMERR_NOERROR, timeBeginPeriod};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CREATESTRUCTW, DefWindowProcW, DispatchMessageW, GWL_USERDATA, GetWindowLongPtrW, MSG,
    PM_REMOVE, PeekMessageW, PostQuitMessage, SetWindowLongPtrW, TranslateMessage, WM_ACTIVATEAPP,
    WM_CLOSE, WM_DESTROY, WM_KEYDOWN, WM_KEYUP, WM_NCCREATE, WM_PAINT, WM_QUIT, WM_SYSKEYDOWN,
    WM_SYSKEYUP,
};
use windows::core::{Error, Result as Win32Result};

const DEFAULT_REFRESH_RATE: u32 = 60;

/// The game updates once every this many monitor refreshes. Keeping the update rate as an
/// exact ratio of the refresh rate, rather than collapsing it to a hertz value up front, lets
/// the audio buffer math stay in integers.
const REFRESHES_PER_UPDATE: u16 = 2;

#[derive(Debug)]
pub enum RecordingState {
    None,
    Recording,
    Playing,
}

#[derive(Debug)]
pub struct Win32Application {
    state: GameState,
    input: InputState,
    window: Win32Window,
    back_buffer: BackBuffer,
    sound_buffer: Option<Vec<StereoSample>>,
    sound_index: Option<u32>,
    sound_safety_margin: Information,
    closing: bool,
    recording_state: RecordingState,
}

impl Win32Application {
    pub fn new() -> Win32Application {
        let window = Win32Window::new();
        Win32Application {
            state: GameState::new(),
            input: InputState::new(),
            window,
            back_buffer: BackBuffer::default(),
            sound_buffer: None,
            sound_index: None,
            sound_safety_margin: Information::zero(),
            closing: false,
            recording_state: RecordingState::None,
        }
    }

    pub fn create_window(&mut self, width: u16, height: u16) -> Result<()> {
        let instance = Self::get_instance()
            .map_err(|e| ApplicationError::wrap("Could not retrieve the Windows handle", e))?;
        let application_pointer = std::ptr::from_mut::<Win32Application>(self).cast::<c_void>();
        self.window
            .create_window(
                instance,
                width,
                height,
                application_pointer,
                Some(window_procedure),
            )
            .map_err(|e| ApplicationError::wrap("Failed to create the window", e))?;
        self.window
            .set_transparency(true)
            .map_err(|e| ApplicationError::wrap("Failed to enable transparency", e))?;

        self.resize_render_buffer()?;

        self.window.draw(&self.back_buffer);

        Ok(())
    }

    fn get_instance() -> Win32Result<HINSTANCE> {
        let instance = unsafe { GetModuleHandleW(None)? };
        Ok(instance.into())
    }

    fn resize_render_buffer(&mut self) -> Result<()> {
        // We capture the actual client rectangle here. The client area is smaller
        // than the window area, typically, so we need the actual dimensions.
        let client_width_i32 = self.window.client_width();
        let client_width = usize::try_from(client_width_i32)
            .map_err(|e| ApplicationError::wrap("The client width did not fit in a usize", e))?;
        let client_height_i32 = self.window.client_height();
        let client_height = usize::try_from(client_height_i32)
            .map_err(|e| ApplicationError::wrap("The client height did not fit in a usize", e))?;

        #[expect(clippy::cast_precision_loss)]
        let width_in_pixels = Length::new::<pixel>(client_width as f32);
        #[expect(clippy::cast_precision_loss)]
        let height_in_pixels = Length::new::<pixel>(client_height as f32);
        self.back_buffer.resize(width_in_pixels, height_in_pixels)?;

        Ok(())
    }

    fn process_windows_message(
        &mut self,
        message: u32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        match message {
            WM_CLOSE | WM_DESTROY => self.prepare_close(),
            WM_ACTIVATEAPP => self
                .window
                .set_transparency(w_param.0 != 0)
                .map_or(LRESULT(0), |()| LRESULT(0)),
            WM_PAINT => {
                self.window.repaint(&self.back_buffer);
                LRESULT(0)
            }
            WM_SYSKEYDOWN | WM_SYSKEYUP | WM_KEYDOWN | WM_KEYUP => {
                self.handle_key_press(w_param, l_param)
            }
            _ => unsafe { DefWindowProcW(self.window.handle(), message, w_param, l_param) },
        }
    }

    fn prepare_close(&mut self) -> LRESULT {
        self.closing = true;
        unsafe { PostQuitMessage(0) };
        LRESULT(0)
    }

    fn handle_key_press(&mut self, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        let win_keyboard = Win32Keyboard::from_params(w_param, l_param);
        let was_down = win_keyboard.was_key_down();
        let is_down = win_keyboard.is_key_down();
        if was_down == is_down {
            // Ignore repeated messages
            return LRESULT(0);
        }

        let keyboard = self.input.keyboard_mut();
        if win_keyboard.is_alt() && win_keyboard.is_f4() {
            // Allow exiting with ALT+F4
            return self.prepare_close();
        } else if win_keyboard.is_w() || win_keyboard.is_up() {
            InputState::track_down(keyboard.up_mut(), is_down);
        } else if win_keyboard.is_a() || win_keyboard.is_left() {
            InputState::track_down(keyboard.left_mut(), is_down);
        } else if win_keyboard.is_s() || win_keyboard.is_down() {
            InputState::track_down(keyboard.down_mut(), is_down);
        } else if win_keyboard.is_d() || win_keyboard.is_right() {
            InputState::track_down(keyboard.right_mut(), is_down);
        } else if win_keyboard.is_q() {
            InputState::track_down(keyboard.left_shoulder_mut(), is_down);
        } else if win_keyboard.is_e() {
            InputState::track_down(keyboard.right_shoulder_mut(), is_down);
        } else if win_keyboard.is_escape() {
            InputState::track_down(keyboard.start_mut(), is_down);
        } else if win_keyboard.is_l() && is_down {
            // Hitting 'L' begins a recording sessions.
            // Hitting 'L' again causes the recording session to end.
            // The recording will play back in an infinite loop until CTRL+L is hit.
            match (&self.recording_state, win_keyboard.is_control()) {
                (RecordingState::None | RecordingState::Playing, false) => {
                    self.recording_state = RecordingState::Recording;
                }
                (RecordingState::Recording, false) => {
                    self.recording_state = RecordingState::Playing;
                }
                (_, true) => {
                    self.recording_state = RecordingState::None;
                    keyboard.clear();
                }
            }
        }
        LRESULT(0)
    }

    pub fn run(&mut self) -> Result<ExitCode> {
        let monitor_refresh_rate = Self::find_monitor_refresh_rate();
        let frame_duration = Self::frame_duration(monitor_refresh_rate);
        self.state.set_frame_duration(frame_duration);
        // Try to set the Windows scheduler granularity to 1ms!
        let is_sleep_granular = unsafe { timeBeginPeriod(1) } == TIMERR_NOERROR;

        let direct_sound = DirectSound::initialize(self.window.handle()).ok();
        let mut sound_buffer = self.create_sound_buffer(direct_sound.as_ref());

        if let Some(ref mut sound_buffer) = sound_buffer {
            sound_buffer.play_looping().unwrap_or(()); // Ignore errors
            self.sound_safety_margin = self.calculate_sound_safety_margin(monitor_refresh_rate);
        }

        let exe_directory = Self::exe_directory()?;
        let mut loader = ApplicationLoader::new(&exe_directory);
        let mut recorder = PlaybackRecorder::new(&exe_directory);
        let mut counter = PerformanceCounter::start();
        loop {
            if let Some(code) = Self::process_message()? {
                return Ok(code);
            }
            if self.closing {
                continue;
            }

            let application = self.load_application(&mut loader)?;

            self.process_recording(&mut recorder);
            self.process_input(application);
            self.render_to_buffer(application);
            self.fill_sound_buffer_if_available(
                application,
                sound_buffer.as_mut(),
                monitor_refresh_rate,
                &counter,
            );

            self.wait_for_framerate(&mut counter, is_sleep_granular);

            self.window.draw(&self.back_buffer);
            self.update_sound_index(sound_buffer.as_ref());
        }
    }

    fn find_monitor_refresh_rate() -> Frequency {
        let default_refresh_rate = Frequency::new::<hertz>(DEFAULT_REFRESH_RATE);
        let Ok(size) = u16::try_from(size_of::<DEVMODEW>()) else {
            return default_refresh_rate;
        };
        let mut mode = DEVMODEW {
            dmSize: size,
            ..DEVMODEW::default()
        };
        let success = unsafe { EnumDisplaySettingsW(None, ENUM_CURRENT_SETTINGS, &raw mut mode) };
        if !success.as_bool() {
            return default_refresh_rate;
        }
        let frequency = mode.dmDisplayFrequency;
        if frequency == 0 || frequency == 1 {
            return default_refresh_rate;
        }
        Frequency::new::<hertz>(frequency)
    }

    /// How long a single game frame lasts.
    ///
    /// The refresh rate counts refreshes per second, so a count of refreshes divided by it is a
    /// duration.
    fn frame_duration(monitor_refresh_rate: Frequency) -> Time {
        // Refresh rates are small whole numbers, so `f32` represents them exactly.
        #[expect(clippy::cast_precision_loss)]
        let monitor_refresh_rate =
            uom::si::f32::Frequency::new::<hertz>(monitor_refresh_rate.get::<hertz>() as f32);
        f32::from(REFRESHES_PER_UPDATE) / monitor_refresh_rate
    }

    fn create_sound_buffer<'a>(
        &self,
        direct_sound: Option<&'a DirectSound>,
    ) -> Option<DirectSoundBuffer<'a>> {
        direct_sound.as_ref().and_then(|ds| {
            let sound_state = self.state.sound();
            let buffer = ds.create_buffer(
                sound_state.channel_count(),
                sound_state.samples_per_second(),
                sound_state.bits_per_sample(),
                sound_state.buffer_size(),
            );
            buffer.ok()
        })
    }

    /// Half a frame of audio, used as the margin the write cursor must stay ahead of playback.
    fn calculate_sound_safety_margin(&self, monitor_refresh_rate: Frequency) -> Information {
        self.sound_bytes_per_frame(monitor_refresh_rate) / 2
    }

    /// Bytes of audio consumed by a single game frame.
    ///
    /// A frame lasts `REFRESHES_PER_UPDATE / monitor_refresh_rate` seconds, so dividing the byte
    /// rate by the refresh rate cancels the per-second term and leaves a byte count. Dividing
    /// last keeps every term an integer, so this needs no float round trip and carries no
    /// rounding error.
    fn sound_bytes_per_frame(&self, monitor_refresh_rate: Frequency) -> Information {
        let bytes_per_second = self.state.sound().bytes_per_second();
        (bytes_per_second * u32::from(REFRESHES_PER_UPDATE) / monitor_refresh_rate).into()
    }

    fn exe_directory() -> Result<PathBuf> {
        let current_exe_path = std::env::current_exe().map_err(|e| {
            ApplicationError::wrap("Failed to retrieve the current executable path", e)
        })?;
        let current_directory = current_exe_path.parent().ok_or_else(|| {
            ApplicationError::new("Failed to retrieve the current executable parent directory")
        })?;
        Ok(current_directory.to_path_buf())
    }

    fn process_message() -> Result<Option<ExitCode>> {
        loop {
            let mut message = MSG::default();
            let message_result = unsafe { PeekMessageW(&raw mut message, None, 0, 0, PM_REMOVE) };
            if message_result.0 < 0 {
                let result = Error::from_thread();
                return Err(ApplicationError::wrap(
                    "Unable to read the next Windows message",
                    result,
                ));
            } else if message_result.as_bool() {
                // There is a message in the queue
                if message.message == WM_QUIT {
                    let code =
                        u8::try_from(message.wParam.0).map_or(ExitCode::FAILURE, ExitCode::from);
                    return Ok(Some(code));
                }
                unsafe {
                    let _ = TranslateMessage(&raw const message);
                    DispatchMessageW(&raw const message);
                };
            } else {
                return Ok(None);
            }
        }
    }

    fn load_application<'a>(
        &mut self,
        loader: &'a mut ApplicationLoader,
    ) -> Result<&'a mut ApplicationStub> {
        let initialize_context = InitializeContext {
            state: &mut self.state,
            back_buffer: &mut self.back_buffer,
        };
        loader.load(initialize_context)
    }

    fn process_recording(&mut self, recorder: &mut PlaybackRecorder) {
        // It seems our audio can't really use playback. The computation of how many bytes
        // to write depends on how fast the previous frame took to generate. Since this will
        // be different each frame, trying to restore the sound theta causes skipping and
        // other sound artifacts. So we just capture theta upfront and restore it after.
        // Hopefully this gets addressed in a later episode.
        if let RecordingState::Playing = self.recording_state {
            if let Some(state) = recorder.playback().unwrap_or_default() {
                (self.input, self.state) = (state.input, state.state);
            } else {
                recorder.reset_playback().unwrap_or_default(); // We miss a frame here
            }
        } else {
            self.poll_all_controller_state();
            if let Ok(client_coordinates) = self.window.client_coordinate() {
                self.capture_mouse_state(client_coordinates)
                    .unwrap_or_default(); // Ignore errors
            }

            if let RecordingState::Recording = self.recording_state {
                recorder
                    .record(&self.input, &self.state)
                    .unwrap_or_default(); // Ignore errors
            }
        }
    }

    fn process_input(&mut self, application: &ApplicationStub) {
        let context = InputContext {
            input: &self.input,
            state: &mut self.state,
        };
        application.process_input(context);
    }

    // NOTE: We probably don't want to call this as part of the main game loop since it
    // can hang the application if the controller is disconnected.
    fn poll_all_controller_state(&mut self) {
        for controller_index in 0..Win32Controller::max_controller_count() {
            let controller = self.input.get_or_insert_controller_mut(controller_index);
            match Win32Controller::from_index(controller_index) {
                Win32ControllerState::Disabled => controller.set_enabled(false),
                Win32ControllerState::Enabled(win32_controller) => {
                    Self::poll_controller_state(controller, &win32_controller);
                }
            }
        }
    }

    fn poll_controller_state(controller: &mut ControllerState, win32_controller: &Win32Controller) {
        ButtonState::track_down(controller.a_mut(), win32_controller.is_a());
        ButtonState::track_down(controller.b_mut(), win32_controller.is_b());
        ButtonState::track_down(controller.x_mut(), win32_controller.is_x());
        ButtonState::track_down(controller.y_mut(), win32_controller.is_y());
        ButtonState::track_down(controller.start_mut(), win32_controller.is_start());
        ButtonState::track_down(controller.back_mut(), win32_controller.is_back());
        ButtonState::track_down(controller.up_mut(), win32_controller.is_dpad_up());
        ButtonState::track_down(controller.down_mut(), win32_controller.is_dpad_down());
        ButtonState::track_down(controller.left_mut(), win32_controller.is_dpad_left());
        ButtonState::track_down(controller.right_mut(), win32_controller.is_dpad_right());
        ButtonState::track_down(
            controller.left_shoulder_mut(),
            win32_controller.is_left_shoulder(),
        );
        ButtonState::track_down(
            controller.right_shoulder_mut(),
            win32_controller.is_right_shoulder(),
        );

        let left_joystick = controller.left_joystick_mut();
        left_joystick.set_x_ratio(win32_controller.left_joystick_x());
        left_joystick.set_y_ratio(win32_controller.left_joystick_y());
        let right_joystick = controller.right_joystick_mut();
        right_joystick.set_x_ratio(win32_controller.right_joystick_x());
        right_joystick.set_y_ratio(win32_controller.right_joystick_y());

        controller.set_left_trigger_ratio(win32_controller.left_trigger());
        controller.set_right_trigger_ratio(win32_controller.right_trigger());
        controller.set_enabled(true);
    }

    fn capture_mouse_state(&mut self, client_coordinate: POINT) -> Win32Result<()> {
        let win32_mouse = Win32Mouse::new();
        let mouse_coordinate = win32_mouse.coordinates()?;
        let mouse = self.input.mouse_mut();
        let x = mouse_coordinate.x().abs_diff(client_coordinate.x);
        let y = mouse_coordinate.y().abs_diff(client_coordinate.y);
        mouse.set_x(x);
        mouse.set_y(y);

        InputState::track_down(mouse.left_mut(), win32_mouse.is_left());
        InputState::track_down(mouse.middle_mut(), win32_mouse.is_middle());
        InputState::track_down(mouse.right_mut(), win32_mouse.is_right());

        Ok(())
    }

    fn render_to_buffer(&mut self, application: &ApplicationStub) {
        let context = RenderContext {
            input: &self.input,
            state: &mut self.state,
            buffer: &mut self.back_buffer,
        };
        application.render(context);
    }

    fn fill_sound_buffer_if_available(
        &mut self,
        application: &mut ApplicationStub,
        sound_buffer: Option<&mut DirectSoundBuffer<'_>>,
        monitor_refresh_rate: Frequency,
        counter: &PerformanceCounter,
    ) {
        let Some(sound_index) = self.sound_index else {
            return;
        };
        let Some(sound_buffer) = sound_buffer else {
            return;
        };
        self.fill_sound_buffer(
            application,
            sound_buffer,
            sound_index,
            monitor_refresh_rate,
            counter,
        );
    }

    fn fill_sound_buffer(
        &mut self,
        application: &mut dyn Application,
        direct_sound_buffer: &mut DirectSoundBuffer<'_>,
        sound_index: u32,
        monitor_refresh_rate: Frequency,
        performance_counter: &PerformanceCounter,
    ) {
        let Ok((play_cursor, write_cursor)) = direct_sound_buffer.get_cursors() else {
            return;
        };
        let play_cursor = Information::new::<byte>(play_cursor);
        let write_cursor = Information::new::<byte>(write_cursor);
        let buffer_length = Information::new::<byte>(direct_sound_buffer.length());
        let bytes_per_sample = self.state.sound().bytes_per_sample();
        // Wrapping the sample index into the buffer before converting it to bytes keeps the
        // offset from overflowing once the index has been running for a few hours.
        let buffer_samples = (buffer_length / bytes_per_sample).value;
        let write_offset = bytes_per_sample * (sound_index % buffer_samples);

        let safe_write_cursor = write_cursor
            .saturating_add(self.sound_safety_margin)
            .saturating_add(if write_cursor < play_cursor {
                buffer_length
            } else {
                Information::zero()
            });
        let frame_time_elapsed =
            Time::new::<second>(performance_counter.metrics().elapsed_time().as_secs_f32());
        let target_frame_duration = self.state.frame_duration();
        let remaining_frame_time = (target_frame_duration - frame_time_elapsed).max(Time::zero());
        let remaining_time_ratio: Ratio = remaining_frame_time / target_frame_duration;
        let bytes_per_frame = self.sound_bytes_per_frame(monitor_refresh_rate);
        // The fraction of the frame still to elapse is genuinely fractional, so this one step
        // stays in floating point. `f64::from` is lossless from both `f32` and `u32`, and the
        // ratio is in [0, 1], so the only thing `as` discards here is the fraction of a byte.
        let remaining_bytes = f64::from(remaining_time_ratio.get::<ratio>())
            * f64::from(bytes_per_frame.get::<byte>());
        #[expect(clippy::cast_sign_loss)]
        #[expect(clippy::cast_possible_truncation)]
        let remaining_bytes = Information::new::<byte>(remaining_bytes as u32);
        let expected_frame_boundary = play_cursor.saturating_add(remaining_bytes);
        let audio_is_latent = safe_write_cursor >= expected_frame_boundary;
        let target_cursor = if audio_is_latent {
            write_cursor
                .saturating_add(self.sound_safety_margin)
                .saturating_add(bytes_per_frame)
        } else {
            expected_frame_boundary.saturating_add(bytes_per_frame)
        };
        let target_cursor = target_cursor % buffer_length;
        let bytes_to_write = match write_offset.cmp(&target_cursor) {
            Ordering::Greater => buffer_length
                .saturating_sub(write_offset)
                .saturating_add(target_cursor),
            Ordering::Less => target_cursor.saturating_sub(write_offset),
            Ordering::Equal => Information::zero(),
        };
        if bytes_to_write == Information::zero() {
            return;
        }

        let sample_count = (bytes_to_write / bytes_per_sample).value;
        let sample_count = usize::try_from(sample_count).unwrap_or(0); // 16-bit OS?
        let buffer_samples = usize::try_from(buffer_samples).unwrap_or(0); // 16-bit OS?
        let sound_buffer = self
            .sound_buffer
            .get_or_insert_with(|| vec![StereoSample::default(); buffer_samples]);
        let sound_buffer = &mut sound_buffer[..sample_count];
        let context = AudioContext {
            state: &mut self.state,
            sound_buffer,
        };
        application.write_sound(context);

        let buffer_lock_guard =
            direct_sound_buffer.lock(write_offset.get::<byte>(), bytes_to_write.get::<byte>());
        let Ok(buffer_lock_guard) = buffer_lock_guard else {
            return;
        };

        Self::copy_sound_buffer(
            buffer_lock_guard.region1(),
            buffer_lock_guard.region1_size(),
            sound_buffer,
            0,
        );

        Self::copy_sound_buffer(
            buffer_lock_guard.region2(),
            buffer_lock_guard.region2_size(),
            sound_buffer,
            buffer_lock_guard.region1_size(),
        );
        let sample_count = u32::try_from(sample_count).unwrap_or(0); // Impossible?
        self.sound_index = Some(sound_index.wrapping_add(sample_count));
    }

    fn copy_sound_buffer(
        destination: *mut c_void,
        destination_length_in_bytes: u32,
        source: &[StereoSample],
        source_offset_in_bytes: u32,
    ) {
        if destination.is_null() {
            return;
        }
        let sample_count =
            usize::try_from(destination_length_in_bytes).unwrap_or(0) / size_of::<StereoSample>();
        let sample_out =
            unsafe { slice::from_raw_parts_mut(destination.cast::<StereoSample>(), sample_count) };
        let source_offset =
            usize::try_from(source_offset_in_bytes).unwrap_or(0) / size_of::<StereoSample>();
        let source_end = source_offset.saturating_add(sample_count);
        let source_slice = &source[source_offset..source_end];
        debug_assert_eq!(source_slice.len(), sample_out.len());
        sample_out.copy_from_slice(source_slice);
    }

    fn get_sample_index(&self, direct_sound_buffer: &DirectSoundBuffer<'_>) -> Option<u32> {
        let (_, write_cursor) = direct_sound_buffer.get_cursors().ok()?;
        let write_cursor = Information::new::<byte>(write_cursor);
        let bytes_per_sample = self.state.sound().bytes_per_sample();
        let index = (write_cursor / bytes_per_sample).value;
        Some(index)
    }

    fn wait_for_framerate(&self, counter: &mut PerformanceCounter, is_sleep_granular: bool) {
        let mut metrics = counter.metrics();
        let mut time_elapsed = metrics.elapsed_time();
        let frame_duration = self.state.frame_duration().get::<second>();
        let frame_duration = Duration::from_secs_f32(frame_duration);
        while time_elapsed < frame_duration {
            if is_sleep_granular {
                let remaining = frame_duration.saturating_sub(time_elapsed);
                std::thread::sleep(remaining);
            }

            metrics = counter.metrics();
            time_elapsed = metrics.elapsed_time();
        }

        counter.restart();
    }

    fn update_sound_index(&mut self, sound_buffer: Option<&DirectSoundBuffer<'_>>) {
        // After a single frame, we have a better idea how far away the sound
        // play cursor is from the write cursor. We initialize the sound index
        // as a flag for sound to start being written now that the metrics are
        // recorded.
        if self.sound_index.is_none()
            && let Some(sound_buffer) = sound_buffer
        {
            self.sound_index = self.get_sample_index(sound_buffer);
        }
    }
}

extern "system" fn window_procedure(
    window_handle: HWND,
    message: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create_struct = unsafe { &*(l_param.0 as *const CREATESTRUCTW) };
        let application = create_struct.lpCreateParams.cast::<Win32Application>();
        unsafe { SetWindowLongPtrW(window_handle, GWL_USERDATA, application as isize) };
        return LRESULT(1); // Indicate we should proceed with creating the window.
    }

    let application_pointer = unsafe { GetWindowLongPtrW(window_handle, GWL_USERDATA) };
    let application_pointer = application_pointer as *mut Win32Application;
    if application_pointer.is_null() {
        // We're not initialized yet, so just let the default handler run.
        return unsafe { DefWindowProcW(window_handle, message, w_param, l_param) };
    }

    // We keep an Application object alive for the duration of the application.
    // This allows us to maintain state about the application without relying on
    // global variables.
    let application = unsafe { &mut *application_pointer };
    if application.window.handle() != window_handle {
        // Some of the messages passed to our application are not directed toward
        // our window. We need to pass through the correct window handle for those
        // messages or the window appears broken! I'll be curious to see if any
        // behavior is broken if we ignore messages directed toward other windows.
        return unsafe { DefWindowProcW(window_handle, message, w_param, l_param) };
    }
    application.process_windows_message(message, w_param, l_param)
}
