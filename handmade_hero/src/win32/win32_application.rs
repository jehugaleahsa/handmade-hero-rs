use super::direct_sound::DirectSound;
use super::direct_sound_buffer::DirectSoundBuffer;
use super::win32_controller::{Win32Controller, Win32ControllerState};
use super::win32_key_event::{self, Win32KeyEvent};
use super::win32_mouse::Win32Mouse;
use super::win32_sound_output::Win32SoundOutput;
use super::win32_window::Win32Window;
use crate::application_loader::{ApplicationLoader, ApplicationStub, LoadedApplication};
use crate::playback_recorder::PlaybackRecorder;
use crate::win32::win32_monitor::find_monitor_refresh_rate;
use handmade_hero_interface::application::Application;
use handmade_hero_interface::application_error::{ApplicationError, Result};
use handmade_hero_interface::audio_context::AudioContext;
use handmade_hero_interface::audio_format::AudioFormat;
use handmade_hero_interface::back_buffer::BackBuffer;
use handmade_hero_interface::controller_state::ControllerState;
use handmade_hero_interface::game_state::GameState;
use handmade_hero_interface::initialize_context::InitializeContext;
use handmade_hero_interface::input_context::InputContext;
use handmade_hero_interface::input_state::InputState;
use handmade_hero_interface::key::Key;
use handmade_hero_interface::key_mapping::KeyMapping;
use handmade_hero_interface::keyboard_state::KeyboardState;
use handmade_hero_interface::performance_counter::PerformanceCounter;
use handmade_hero_interface::plugin_state::PluginState;
use handmade_hero_interface::render_context::RenderContext;
use handmade_hero_interface::sound_buffer::SoundBuffer;
use handmade_hero_interface::units::si::frequency::Frequency;
use handmade_hero_interface::units::si::information::Information;
use handmade_hero_interface::units::si::length::pixel;
use std::cmp::Ordering;
use std::ffi::c_void;
use std::path::Path;
use std::process::ExitCode;
use std::rc::Rc;
use std::time::Duration;
use uom::num::Zero;
use uom::si::f32::{Ratio, Time};
use uom::si::frequency::hertz;
use uom::si::information::byte;
use uom::si::length::Length;
use uom::si::ratio::ratio;
use uom::si::time::second;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CREATESTRUCTW, DefWindowProcW, DispatchMessageW, GWL_USERDATA, GetWindowLongPtrW, MSG,
    PM_REMOVE, PeekMessageW, PostQuitMessage, SetWindowLongPtrW, TranslateMessage, WM_ACTIVATEAPP,
    WM_CLOSE, WM_DESTROY, WM_KEYDOWN, WM_KEYUP, WM_KILLFOCUS, WM_NCCREATE, WM_PAINT, WM_QUIT,
    WM_SETFOCUS, WM_SYSKEYDOWN, WM_SYSKEYUP,
};
use windows::core::{Error, Result as Win32Result};

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
    plugin_state: Option<Box<dyn PluginState>>,
    keyboard: KeyboardState,
    key_mapping: KeyMapping,
    window: Win32Window,
    back_buffer: BackBuffer,
    /// Storage the game fills with a frame of audio.
    sound_buffer: SoundBuffer,
    recording_state: RecordingState,
    recorder: PlaybackRecorder,
}

impl Win32Application {
    pub fn new(exe_directory: &Path) -> Win32Application {
        Win32Application {
            state: GameState::new(),
            input: InputState::new(),
            plugin_state: None,
            keyboard: KeyboardState::new(),
            key_mapping: KeyMapping::default(),
            window: Win32Window::new(),
            back_buffer: BackBuffer::default(),
            sound_buffer: SoundBuffer::new(),
            recording_state: RecordingState::None,
            recorder: PlaybackRecorder::new(exe_directory),
        }
    }

    fn create_window(&mut self, title: &str, width: u16, height: u16) -> Result<()> {
        let instance = Self::get_instance()
            .map_err(|e| ApplicationError::wrap("Could not retrieve the Windows handle", e))?;
        let application_pointer = std::ptr::from_mut::<Win32Application>(self).cast::<c_void>();
        self.window
            .create_window(
                instance,
                title,
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
            WM_CLOSE | WM_DESTROY => Self::emit_quitting(),
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
            WM_KILLFOCUS => {
                // Any key still held will be released into some other window, so we would never
                // hear about it. Let go of everything now rather than leave keys stuck down.
                self.keyboard.release_all();
                LRESULT(0)
            }
            WM_SETFOCUS => {
                self.synchronize_keyboard();
                LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(self.window.handle(), message, w_param, l_param) },
        }
    }

    /// Reconciles the tracked keyboard with what is physically held, for when focus returns.
    ///
    /// A user who ALT+TABs away and comes back with a key already down would otherwise have to
    /// release and re-press it before the game noticed. Errors are ignored: the fallback is the
    /// stale-but-harmless state we already had.
    fn synchronize_keyboard(&mut self) {
        if let Ok(key_states) = win32_key_event::physical_key_states() {
            self.keyboard.synchronize(&self.key_mapping, key_states);
        }
    }

    fn emit_quitting() -> LRESULT {
        unsafe { PostQuitMessage(0) };
        LRESULT(0)
    }

    /// Translates one key message into the platform-agnostic keyboard.
    ///
    /// This is deliberately the only place Windows key codes appear. Which button a key drives,
    /// and what happens when several keys drive the same button, is the keyboard's business.
    fn handle_key_press(&mut self, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        let key_event = Win32KeyEvent::from_params(w_param, l_param);
        if key_event.is_repeat() {
            // A held key autorepeats as a stream of key-down messages. Only real changes matter.
            return LRESULT(0);
        }
        let Some(key) = key_event.key() else {
            return LRESULT(0);
        };
        let is_down = key_event.is_down();
        self.keyboard.track_key(&self.key_mapping, key, is_down);

        // Allow exiting with ALT+F4. Handling WM_SYSKEYDOWN ourselves means Windows no longer
        // does this for us.
        if key == Key::F4 && is_down && self.keyboard.is_alt_down() {
            return Self::emit_quitting();
        }
        LRESULT(0)
    }

    /// Hitting 'L' begins a recording session. Hitting 'L' again ends it and starts looping
    /// playback. CTRL+L stops playback and returns to live input.
    ///
    /// Toggling is a discrete action, so it asks whether L *was pressed* this frame rather than
    /// whether it is down. Checking `ended_down` would flip the state on every frame the key is
    /// held. Doing this once per frame instead of inside the message handler also means a tap
    /// that lands entirely between two frames still toggles, thanks to the half-transition count.
    fn process_recording_hotkey(&mut self) {
        if !self.keyboard.key(Key::L).was_pressed() {
            return;
        }
        self.recording_state = match (&self.recording_state, self.keyboard.is_control_down()) {
            (_, true) => RecordingState::None,
            (RecordingState::None | RecordingState::Playing, false) => RecordingState::Recording,
            (RecordingState::Recording, false) => RecordingState::Playing,
        };
    }

    pub fn run(
        &mut self,
        application_loader: &mut ApplicationLoader,
        width: u16,
        height: u16,
    ) -> Result<ExitCode> {
        let monitor_refresh_rate = find_monitor_refresh_rate();
        self.start_application(application_loader, monitor_refresh_rate, width, height)?;

        let direct_sound = DirectSound::initialize(self.window.handle()).ok();
        // The format the device was last asked to open, whether or not it succeeded.
        let mut requested_format = self.state.audio().format();
        let mut sound_output = self.start_sound_output(direct_sound.as_ref(), monitor_refresh_rate);

        let mut counter = PerformanceCounter::start();
        loop {
            // A new frame starts with every half-transition count at zero, while each button
            // keeps whether it ended the last frame down.
            self.input.reset_counts();
            self.keyboard.reset_counts();
            if let Some(code) = Self::process_message()? {
                if let Some(ref mut sound_output) = sound_output {
                    sound_output.stop();
                }
                return Ok(code);
            }
            self.process_recording_hotkey();

            let application = self.load_application(application_loader)?;

            self.process_recording(application.as_ref());
            self.process_input(application.as_ref());
            self.render_to_buffer(application.as_ref());

            let desired_format = self.state.audio().format();
            if desired_format != requested_format {
                if let Some(mut old_output) = sound_output.take() {
                    old_output.stop();
                }
                requested_format = desired_format;
                sound_output = self.start_sound_output(direct_sound.as_ref(), monitor_refresh_rate);
            }

            if let Some(ref mut sound_output) = sound_output {
                self.fill_sound_buffer(application.as_ref(), sound_output, &counter);
            }

            self.wait_for_framerate(&mut counter);

            self.window.draw(&self.back_buffer);
            if let Some(ref mut sound_output) = sound_output {
                sound_output.seed_write_offset();
                if let Ok((play_cursor, write_cursor)) = sound_output.buffer().get_cursors() {
                    let audio = self.state.audio_mut();
                    let flip_play_cursor = usize::try_from(play_cursor).unwrap_or_default();
                    let flip_write_cursor = usize::try_from(write_cursor).unwrap_or_default();
                    audio.set_flip_play_cursor(flip_play_cursor);
                    audio.set_flip_write_cursor(flip_write_cursor);
                }
            }
        }
    }

    /// How long a single game frame lasts.
    ///
    /// The refresh rate counts refreshes per second, so a count of refreshes divided by it is a
    /// duration.
    fn frame_duration(monitor_refresh_rate: Frequency) -> Time {
        // Refresh rates are small whole numbers, so `f32` represents them exactly.
        let refresh_rate = monitor_refresh_rate.get::<hertz>();
        #[expect(clippy::cast_precision_loss)]
        let refresh_rate = uom::si::f32::Frequency::new::<hertz>(refresh_rate as f32);
        f32::from(REFRESHES_PER_UPDATE) / refresh_rate
    }

    /// Opens the audio device with the format the game currently wants and starts it playing.
    ///
    /// `None` when there is no device, or when the device refuses the format. Sound stays off
    /// until the game asks for a different format.
    fn start_sound_output<'a>(
        &self,
        direct_sound: Option<&'a DirectSound>,
        monitor_refresh_rate: Frequency,
    ) -> Option<Win32SoundOutput<'a>> {
        let direct_sound = direct_sound?;
        let format = self.state.audio().format();
        let frame_size = Self::sample_size_per_frame(format, monitor_refresh_rate);
        Win32SoundOutput::start(direct_sound, format, frame_size).ok()
    }

    /// Bytes of audio consumed by a single game frame.
    ///
    /// A frame lasts `REFRESHES_PER_UPDATE / monitor_refresh_rate` seconds, so dividing the byte
    /// rate by the refresh rate cancels the per-second term and leaves a byte count. Dividing
    /// last keeps every term an integer, so this needs no float round trip and carries no
    /// rounding error.
    fn sample_size_per_frame(format: AudioFormat, monitor_refresh_rate: Frequency) -> Information {
        (format.sample_rate() * u32::from(REFRESHES_PER_UPDATE) / monitor_refresh_rate).into()
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

    /// Loads the plugin, creates the window, and starts a fresh game, in that order.
    /// The plugin is unloaded at the end and then reloaded inside the game loop - we
    /// eat the cost, which won't be noticeable at start up.
    ///
    /// The plugin comes first because the window title is the game's name. The window
    /// is created and the frame duration is set before initializing the plugin because
    /// it might need to inspect the render or audio buffers.
    ///
    /// NOTE: The window size and client size are not the same. The client size will be
    /// smaller than the request window size.
    fn start_application(
        &mut self,
        loader: &mut ApplicationLoader,
        monitor_refresh_rate: Frequency,
        width: u16,
        height: u16,
    ) -> Result<()> {
        let LoadedApplication::Fresh(application) = loader.load(&mut self.plugin_state)? else {
            return Err(ApplicationError::new(
                "The plugin was already running before the game started",
            ));
        };

        self.create_window(&application.name(), width, height)?;

        let frame_duration = Self::frame_duration(monitor_refresh_rate);
        self.state.set_frame_duration(frame_duration);

        self.initialize_application(application.as_ref());
        Ok(())
    }

    /// Returns the plugin for this frame. The loader handles hot reloading and carrying the game
    /// state across it.
    fn load_application(&mut self, loader: &mut ApplicationLoader) -> Result<Rc<ApplicationStub>> {
        let loaded_application = loader.load(&mut self.plugin_state)?;
        match loaded_application {
            LoadedApplication::Running(application) => Ok(application),
            LoadedApplication::Fresh(application) => {
                self.initialize_application(application.as_ref());
                Ok(application)
            }
        }
    }

    /// Starts a brand new game with the given plugin.
    fn initialize_application(&mut self, application: &ApplicationStub) {
        let plugin = self.plugin_state.insert(application.create_plugin_state());
        let initialize_context = InitializeContext {
            game_state: &mut self.state,
            plugin_state: plugin.as_mut(),
            back_buffer: &mut self.back_buffer,
        };
        application.initialize(initialize_context);
    }

    fn process_recording(&mut self, application: &ApplicationStub) {
        // It seems our audio can't really use playback. The computation of how many bytes
        // to write depends on how fast the previous frame took to generate. Since this will
        // be different each frame, trying to restore the sound theta causes skipping and
        // other sound artifacts. So we just capture theta upfront and restore it after.
        // Hopefully this gets addressed in a later episode.
        if let RecordingState::Playing = self.recording_state {
            if let Some(playback) = self.recorder.playback(application).unwrap_or_default() {
                // Assigning drops the previous plugin state.
                self.input = playback.input;
                self.state = playback.state;
                self.plugin_state = Some(playback.plugin);
            } else {
                self.recorder.reset_playback().unwrap_or_default(); // We miss a frame here
            }
        } else {
            // The keyboard has been accumulating key events all frame. Publish a copy as the
            // input the game sees. Because this happens every live frame, stopping playback needs
            // no special reset: the next frame simply shows the real keys again.
            *self.input.keyboard_mut() = self.keyboard.clone();
            self.poll_all_controller_state();
            if let Ok(client_coordinates) = self.window.client_coordinate() {
                self.capture_mouse_state(client_coordinates)
                    .unwrap_or_default(); // Ignore errors
            }

            if let RecordingState::Recording = self.recording_state
                && let Some(plugin) = self.plugin_state.as_deref()
            {
                self.recorder
                    .record(&self.input, &self.state, plugin)
                    .unwrap_or_default(); // Ignore errors
            }
        }
    }

    fn process_input(&mut self, application: &ApplicationStub) {
        let Some(plugin) = self.plugin_state.as_deref_mut() else {
            return;
        };
        let context = InputContext {
            input_state: &self.input,
            game_state: &mut self.state,
            plugin_state: plugin,
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
        controller.a_mut().track_down(win32_controller.is_a());
        controller.b_mut().track_down(win32_controller.is_b());
        controller.x_mut().track_down(win32_controller.is_x());
        controller.y_mut().track_down(win32_controller.is_y());
        controller
            .start_mut()
            .track_down(win32_controller.is_start());
        controller.back_mut().track_down(win32_controller.is_back());
        controller
            .up_mut()
            .track_down(win32_controller.is_dpad_up());
        controller
            .down_mut()
            .track_down(win32_controller.is_dpad_down());
        controller
            .left_mut()
            .track_down(win32_controller.is_dpad_left());
        controller
            .right_mut()
            .track_down(win32_controller.is_dpad_right());
        controller
            .left_shoulder_mut()
            .track_down(win32_controller.is_left_shoulder());
        controller
            .right_shoulder_mut()
            .track_down(win32_controller.is_right_shoulder());

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
        let Some(plugin) = self.plugin_state.as_deref_mut() else {
            return;
        };
        let context = RenderContext {
            game_state: &mut self.state,
            plugin_state: plugin,
            input_state: &self.input,
            buffer: &mut self.back_buffer,
        };
        application.render(context);
    }

    fn fill_sound_buffer(
        &mut self,
        application: &ApplicationStub,
        sound_output: &mut Win32SoundOutput<'_>,
        performance_counter: &PerformanceCounter,
    ) {
        let Some(write_offset) = sound_output.write_offset() else {
            return; // The device hasn't reported a write cursor to start from yet.
        };
        let Ok((play_cursor, write_cursor)) = sound_output.buffer().get_cursors() else {
            return;
        };
        let expected_frame_boundary =
            self.find_expected_frame_boundary(sound_output, performance_counter, play_cursor);
        let target_cursor = Self::find_target_cursor(
            sound_output,
            play_cursor,
            write_cursor,
            expected_frame_boundary,
        );
        let write_size = self.find_write_size(sound_output.buffer(), write_offset, target_cursor);
        if write_size == Information::zero() {
            return;
        }

        let buffer_length = sound_output.buffer().length();
        let audio_state = self.state.audio_mut();
        audio_state.set_buffer_length(buffer_length);
        let play_cursor = usize::try_from(play_cursor).unwrap_or_default();
        audio_state.set_output_play_cursor(play_cursor);
        let write_cursor = usize::try_from(write_cursor).unwrap_or_default();
        audio_state.set_output_write_cursor(write_cursor);
        let write_offset_usize = usize::try_from(write_offset).unwrap_or_default();
        audio_state.set_output_write_offset(write_offset_usize);
        audio_state.set_output_write_length(write_size);
        let expected_frame_boundary_usize =
            usize::try_from(expected_frame_boundary).unwrap_or_default();
        audio_state.set_expected_frame_boundary(expected_frame_boundary_usize);

        let Some(sound_bytes) = self.write_sound(application, write_size, buffer_length) else {
            return;
        };
        Self::copy_sound_buffer(
            sound_output.buffer_mut(),
            write_offset,
            write_size,
            sound_bytes,
        );

        let next_write_offset =
            Self::find_next_write_offset(write_offset, write_size, buffer_length);
        sound_output.set_write_offset(next_write_offset);
    }

    /// Bytes to write to carry the buffer from `write_offset` around to `target_cursor`.
    fn find_write_size(
        &self,
        direct_sound_buffer: &DirectSoundBuffer<'_>,
        write_offset: u32,
        target_cursor: u32,
    ) -> Information {
        let buffer_length = direct_sound_buffer.length().get::<byte>();
        let bytes_to_write = match write_offset.cmp(&target_cursor) {
            Ordering::Greater => buffer_length
                .saturating_sub(write_offset)
                .saturating_add(target_cursor),
            Ordering::Less => target_cursor.saturating_sub(write_offset),
            Ordering::Equal => 0,
        };
        // The target cursor is estimated from elapsed time, so it can land partway through a
        // sample. Rounding down keeps every write offset on a sample boundary, which the ring
        // buffer math cannot guarantee on its own since offsets are bytes. The stray bytes are
        // covered by the next frame's write.
        let sample_size = self.state.audio().format().sample_size().get::<byte>();
        let misaligned_bytes = bytes_to_write.checked_rem(sample_size).unwrap_or(0);
        let aligned_bytes_to_write = bytes_to_write.saturating_sub(misaligned_bytes);
        Information::new::<byte>(aligned_bytes_to_write)
    }

    /// Where we start writing and how much we write depends on the audio latency.
    /// We start the audio playing against an empty sound buffer during the first game loop.
    /// From that, we can inspect the audio latency (distance between the play and write cursor).
    /// If the latency is high, approximately the same length as our frame rate or greater, we
    /// write out a full frame's worth of audio, plus a safety margin. For low latency audio,
    /// we write out a full frame's worth of audio, plus however much audio is left from the
    /// play cursor to the end of the current frame.
    fn find_target_cursor(
        sound_output: &Win32SoundOutput<'_>,
        play_cursor: u32,
        write_cursor: u32,
        expected_frame_boundary: u32,
    ) -> u32 {
        let safety_margin = sound_output.safety_margin();
        let safe_write_cursor = write_cursor.saturating_add(safety_margin.get::<byte>());
        let buffer_length = sound_output.buffer().length();
        let buffer_length_bytes = buffer_length.get::<byte>();
        let mut normalized_safe_write_cursor = safe_write_cursor;
        if write_cursor < play_cursor {
            normalized_safe_write_cursor += buffer_length_bytes;
        }
        let audio_is_latent = normalized_safe_write_cursor >= expected_frame_boundary;

        let frame_size = sound_output.frame_size();
        let frame_size_bytes = frame_size.get::<byte>();
        let target_cursor = if audio_is_latent {
            safe_write_cursor.saturating_add(frame_size_bytes)
        } else {
            expected_frame_boundary.saturating_add(frame_size_bytes)
        };
        target_cursor % buffer_length_bytes
    }

    fn find_expected_frame_boundary(
        &self,
        sound_output: &Win32SoundOutput<'_>,
        performance_counter: &PerformanceCounter,
        play_cursor: u32,
    ) -> u32 {
        let frame_time_elapsed =
            Time::new::<second>(performance_counter.metrics().elapsed_time().as_secs_f32());
        let target_frame_duration = self.state.frame_duration();
        let remaining_frame_time = (target_frame_duration - frame_time_elapsed).max(Time::zero());
        let remaining_time_ratio: Ratio = remaining_frame_time / target_frame_duration;
        let frame_size = sound_output.frame_size();
        // The fraction of the frame still to elapse is genuinely fractional, so this one step
        // stays in floating point. `f64::from` is lossless from both `f32` and `u32`, and the
        // ratio is in [0, 1], so the only thing `as` discards here is the fraction of a byte.
        let remaining_bytes =
            f64::from(remaining_time_ratio.get::<ratio>()) * f64::from(frame_size.get::<byte>());
        #[expect(clippy::cast_sign_loss)]
        #[expect(clippy::cast_possible_truncation)]
        let remaining_bytes = remaining_bytes as u32;
        play_cursor.saturating_add(remaining_bytes)
    }

    /// Has the game fill the next `write_size` bytes of audio and returns them.
    fn write_sound(
        &mut self,
        application: &ApplicationStub,
        write_size: Information,
        buffer_length: Information,
    ) -> Option<&[u8]> {
        let plugin = self.plugin_state.as_deref_mut()?;
        self.sound_buffer.ensure_capacity(buffer_length);
        let sample_size = self.state.audio().format().sample_size();
        let window = self.sound_buffer.window(write_size, sample_size)?;

        let context = AudioContext {
            game_state: &mut self.state,
            plugin_state: plugin,
            input_state: &self.input,
            sound_buffer: window,
        };
        application.write_sound(context);

        self.sound_buffer.bytes(write_size)
    }

    fn copy_sound_buffer(
        direct_sound_buffer: &mut DirectSoundBuffer<'_>,
        write_offset: u32,
        write_size: Information,
        sound_bytes: &[u8],
    ) {
        let buffer_lock_guard = direct_sound_buffer.lock(write_offset, write_size);
        let Ok(mut buffer_lock_guard) = buffer_lock_guard else {
            return;
        };
        buffer_lock_guard.copy_from(sound_bytes);
    }

    fn find_next_write_offset(
        write_offset: u32,
        write_size: Information,
        buffer_length: Information,
    ) -> u32 {
        // Safety: The maximum DirectSound buffer is less than u32::MAX, so overflow isn't possible.
        // A single write never covers the whole buffer, so the advanced offset wraps at most once.
        let buffer_length = buffer_length.get::<byte>();
        let mut next_offset = write_offset.strict_add(write_size.get::<byte>());
        if next_offset >= buffer_length {
            next_offset -= buffer_length;
        }
        next_offset
    }

    fn wait_for_framerate(&self, counter: &mut PerformanceCounter) {
        let mut metrics = counter.metrics();
        let mut time_elapsed = metrics.elapsed_time();
        let frame_duration = self.state.frame_duration().get::<second>();
        let frame_duration = Duration::from_secs_f32(frame_duration);
        while time_elapsed < frame_duration {
            let remaining = frame_duration.saturating_sub(time_elapsed);
            std::thread::sleep(remaining);

            metrics = counter.metrics();
            time_elapsed = metrics.elapsed_time();
        }

        counter.restart();
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
        // The default handler is what records the window title, so it still has to run.
        return unsafe { DefWindowProcW(window_handle, message, w_param, l_param) };
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
