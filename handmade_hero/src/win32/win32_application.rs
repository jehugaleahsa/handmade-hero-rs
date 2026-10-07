use super::direct_sound::DirectSound;
use super::direct_sound_buffer::DirectSoundBuffer;
use super::win32_controller::{Win32Controller, Win32ControllerState};
use super::win32_sound_output::Win32SoundOutput;
use super::win32_state::{Win32State, window_procedure};
use super::win32_window::Win32Window;
use crate::application_loader::{ApplicationLoader, ApplicationStub, LoadedApplication};
use crate::playback_recorder::PlaybackRecorder;
use crate::win32::win32_monitor::{
    Win32Monitor, find_monitors, restore_display_mode, set_display_mode, set_dpi_awareness,
};
use handmade_hero_interface::application::Application;
use handmade_hero_interface::application_error::{ApplicationError, Result};
use handmade_hero_interface::audio_context::AudioContext;
use handmade_hero_interface::audio_format::AudioFormat;
use handmade_hero_interface::back_buffer::BackBuffer;
use handmade_hero_interface::controller_state::ControllerState;
use handmade_hero_interface::display::Display;
use handmade_hero_interface::display_settings::DisplaySettings;
use handmade_hero_interface::display_state::DisplayState;
use handmade_hero_interface::game_state::GameState;
use handmade_hero_interface::initialize_context::InitializeContext;
use handmade_hero_interface::input_context::InputContext;
use handmade_hero_interface::input_state::InputState;
use handmade_hero_interface::key::Key;
use handmade_hero_interface::monitor::Monitor;
use handmade_hero_interface::monitor_mode::MonitorMode;
use handmade_hero_interface::performance_counter::PerformanceCounter;
use handmade_hero_interface::plugin_state::PluginState;
use handmade_hero_interface::render_context::RenderContext;
use handmade_hero_interface::sound_buffer::SoundBuffer;
use handmade_hero_interface::units::si::information::Information;
use handmade_hero_interface::units::si::length::pixel;
use std::ffi::c_void;
use std::path::Path;
use std::process::ExitCode;
use std::rc::Rc;
use std::time::Duration;
use uom::num::Zero;
use uom::si::f32::{Ratio, Time};
use uom::si::frequency::hertz;
use uom::si::information::byte;
use uom::si::information_rate::byte_per_second;
use uom::si::length::Length;
use uom::si::ratio::ratio;
use uom::si::time::second;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage, WM_QUIT,
};
use windows::core::{Error, Result as Win32Result};

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
    window: Win32Window,
    back_buffer: BackBuffer,
    /// Storage the game fills with a frame of audio.
    sound_buffer: SoundBuffer,
    recording_state: RecordingState,
    recorder: PlaybackRecorder,
    monitors: Vec<Win32Monitor>,
    monitors_changed: bool,
    display: DisplayState,
    mode_change: ModeChange,
}

/// Whether the fullscreen monitor is in a display mode the game switched it to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModeChange {
    /// Every monitor is in the mode the player chose in Windows.
    None,
    /// The fullscreen monitor is in the game's mode.
    Active,
    /// The player switched to another application, so their own mode is back until the game is
    /// active again.
    Suspended,
}

impl Win32Application {
    pub fn new(exe_directory: &Path) -> Win32Application {
        Win32Application {
            state: GameState::new(),
            input: InputState::new(),
            plugin_state: None,
            window: Win32Window::new(),
            back_buffer: BackBuffer::default(),
            sound_buffer: SoundBuffer::new(),
            recording_state: RecordingState::None,
            recorder: PlaybackRecorder::new(exe_directory),
            monitors: Vec::new(),
            monitors_changed: false,
            display: DisplayState::new(),
            mode_change: ModeChange::None,
        }
    }

    fn create_window(
        &mut self,
        state: &Win32State,
        title: &str,
        width: u16,
        height: u16,
    ) -> Result<()> {
        let instance = Self::get_instance()
            .map_err(|e| ApplicationError::wrap("Could not retrieve the Windows handle", e))?;
        let state_pointer = std::ptr::from_ref(state).cast::<c_void>();
        self.window
            .create_window(
                instance,
                title,
                width,
                height,
                state_pointer,
                Some(window_procedure),
            )
            .map_err(|e| ApplicationError::wrap("Failed to create the window", e))?;
        Win32Window::set_transparency(self.window.handle(), true)
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
        let client_size = self
            .window
            .client_size()
            .map_err(|e| ApplicationError::wrap("Could not read the window's client size", e))?;
        let client_width = usize::try_from(client_size.cx)
            .map_err(|e| ApplicationError::wrap("The client width did not fit in a usize", e))?;
        let client_height = usize::try_from(client_size.cy)
            .map_err(|e| ApplicationError::wrap("The client height did not fit in a usize", e))?;
        if client_width == 0 || client_height == 0 {
            // A minimized window has no client area. Keep the buffer for when it comes back.
            return Ok(());
        }

        #[expect(clippy::cast_precision_loss)]
        let width_in_pixels = Length::new::<pixel>(client_width as f32);
        #[expect(clippy::cast_precision_loss)]
        let height_in_pixels = Length::new::<pixel>(client_height as f32);
        self.back_buffer.resize(width_in_pixels, height_in_pixels)?;

        Ok(())
    }

    fn refresh_monitors(&mut self) {
        self.monitors = find_monitors(self.window.handle());
        self.monitors_changed = true;
    }

    fn game_monitors(&self) -> Vec<Monitor> {
        self.monitors
            .iter()
            .map(Win32Monitor::monitor)
            .cloned()
            .collect()
    }

    /// Hitting 'L' begins a recording session. Hitting 'L' again ends it and starts looping
    /// playback. CTRL+L stops playback and returns to live input.
    ///
    /// Toggling is a discrete action, so it asks whether L *was pressed* this frame rather than
    /// whether it is down. Checking `ended_down` would flip the state on every frame the key is
    /// held. Doing this once per frame instead of inside the message handler also means a tap
    /// that lands entirely between two frames still toggles, thanks to the half-transition count.
    fn process_recording_hotkey(&mut self, state: &Win32State) {
        let keyboard = state.keyboard();
        if !keyboard.key(Key::L).was_pressed() {
            return;
        }
        self.recording_state = match (&self.recording_state, keyboard.is_control_down()) {
            (_, true) => RecordingState::None,
            (RecordingState::None | RecordingState::Playing, false) => RecordingState::Recording,
            (RecordingState::Recording, false) => RecordingState::Playing,
        };
    }

    /// Applies the display settings the game asked for during the previous frame.
    fn process_display_request(&mut self) {
        let Some(request) = self.display.take_request() else {
            return;
        };
        if let RecordingState::Playing = self.recording_state {
            // A request replayed from a recording would change the display on every loop.
            return;
        }
        let applied = self.apply_display_settings(&request);
        if applied.is_ok() {
            self.display.set_current(request);
        }
        // A request that failed partway may still have changed the window's size, so the buffer
        // follows the window either way.
        let result = applied.and(self.resize_render_buffer());
        self.display.set_last_request(result);
    }

    fn apply_display_settings(&mut self, settings: &DisplaySettings) -> Result<()> {
        // A monitor the game switched to another mode goes back to the player's own mode, unless
        // the new settings pick a mode for that same monitor.
        if let DisplaySettings::Fullscreen {
            monitor_identifier: current,
            ..
        } = self.display.current()
        {
            let stays_on_monitor = if let DisplaySettings::Fullscreen {
                monitor_identifier: next,
                ..
            } = settings
            {
                next == current
            } else {
                false
            };
            if !stays_on_monitor {
                // A suspended change has already given the player their mode back.
                if self.mode_change == ModeChange::Active
                    && let Some(monitor) = self.find_monitor(current)
                {
                    restore_display_mode(monitor)?;
                }
                self.mode_change = ModeChange::None;
            }
        }
        match settings {
            DisplaySettings::Windowed => self
                .window
                .exit_fullscreen()
                .map_err(|e| ApplicationError::wrap("Could not leave fullscreen", e)),
            DisplaySettings::Fullscreen {
                monitor_identifier,
                mode,
            } => self.enter_fullscreen(monitor_identifier, mode),
        }
    }

    fn enter_fullscreen(&mut self, monitor_identifier: &str, mode: &MonitorMode) -> Result<()> {
        let monitor = self
            .find_monitor(monitor_identifier)
            .ok_or_else(|| ApplicationError::new("The requested monitor is not connected"))?;
        let mut bounds = monitor.bounds();
        let is_current_mode = monitor
            .monitor()
            .current_mode()
            .is_some_and(|current| current.matches(mode));
        if !is_current_mode {
            set_display_mode(monitor, mode)?;
            self.mode_change = ModeChange::Active;
            // A new resolution changes the monitor's size, so its bounds have to be read again.
            self.refresh_monitors();
            bounds = self
                .find_monitor(monitor_identifier)
                .ok_or_else(|| ApplicationError::new("The requested monitor is not connected"))?
                .bounds();
        }
        self.window
            .enter_fullscreen(&bounds)
            .map_err(|e| ApplicationError::wrap("Could not enter fullscreen", e))
    }

    fn process_activation(&mut self, state: &Win32State) {
        let Some(is_active) = state.take_activation() else {
            return;
        };
        let DisplaySettings::Fullscreen {
            monitor_identifier,
            mode,
        } = self.display.current().clone()
        else {
            return;
        };
        match (is_active, self.mode_change) {
            (false, ModeChange::Active) => {
                if let Some(monitor) = self.find_monitor(&monitor_identifier) {
                    let _ = restore_display_mode(monitor);
                }
                self.mode_change = ModeChange::Suspended;
                self.refresh_monitors();
                self.window.minimize();
            }
            (true, ModeChange::Suspended) => {
                // Windows activates a minimized window before restoring it, so the window may
                // still be minimized here. Restoring it first gives the repositioning and the
                // buffer resize a real window to work with.
                self.window.restore();
                // The player's mode is back, so this switches modes again and repositions the
                // window to the monitor's new size.
                let _ = self
                    .enter_fullscreen(&monitor_identifier, &mode)
                    .and_then(|()| self.resize_render_buffer());
                // Switching modes marks the change active again. If no switch happened, because
                // the monitor was already in that mode or switching failed, nothing is changed.
                if self.mode_change == ModeChange::Suspended {
                    self.mode_change = ModeChange::None;
                }
            }
            _ => {}
        }
    }

    fn find_monitor(&self, identifier: &str) -> Option<&Win32Monitor> {
        self.monitors
            .iter()
            .find(|monitor| monitor.monitor().identifier() == identifier)
    }

    pub fn run(
        &mut self,
        application_loader: &mut ApplicationLoader,
        width: u16,
        height: u16,
    ) -> Result<ExitCode> {
        let _ = set_dpi_awareness();
        let state = Win32State::new();
        let result = self.run_game(&state, application_loader, width, height);
        self.window.destroy();
        result
    }

    fn run_game(
        &mut self,
        state: &Win32State,
        application_loader: &mut ApplicationLoader,
        width: u16,
        height: u16,
    ) -> Result<ExitCode> {
        self.start_application(state, application_loader, width, height)?;

        let direct_sound = DirectSound::initialize(self.window.handle()).ok();
        // The format the device was last asked to open, whether or not it succeeded.
        let mut requested_format = self.state.audio().format();
        let mut sound_output = self.start_sound_output(direct_sound.as_ref());

        let mut counter = PerformanceCounter::start();
        loop {
            // A new frame starts with every half-transition count at zero, while each button
            // keeps whether it ended the last frame down.
            self.input.reset_counts();
            state.reset_counts();
            if let Some(code) = Self::process_message()? {
                if let Some(ref mut sound_output) = sound_output {
                    sound_output.stop();
                }
                return Ok(code);
            }
            if state.take_monitors_changed() {
                self.refresh_monitors();
            }
            self.process_recording_hotkey(state);
            self.process_activation(state);
            self.process_display_request();

            let application = self.load_application(application_loader)?;
            if self.monitors_changed {
                self.handle_monitor_change(application.as_ref(), sound_output.as_mut());
            }

            self.process_recording(state, application.as_ref());
            self.process_input(application.as_ref());
            self.render_to_buffer(application.as_ref());

            let desired_format = self.state.audio().format();
            if desired_format != requested_format {
                if let Some(mut old_output) = sound_output.take() {
                    old_output.stop();
                }
                requested_format = desired_format;
                sound_output = self.start_sound_output(direct_sound.as_ref());
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

    /// Opens the audio device with the format the game currently wants and starts it playing.
    ///
    /// `None` when there is no device, or when the device refuses the format. Sound stays off
    /// until the game asks for a different format.
    fn start_sound_output<'a>(
        &self,
        direct_sound: Option<&'a DirectSound>,
    ) -> Option<Win32SoundOutput<'a>> {
        let direct_sound = direct_sound?;
        let format = self.state.audio().format();
        let frame_size = self.sample_size_per_frame(format);
        Win32SoundOutput::start(direct_sound, format, frame_size).ok()
    }

    fn sample_size_per_frame(&self, format: AudioFormat) -> Information {
        let bytes_per_second = format.sample_rate().get::<byte_per_second>();
        #[expect(clippy::cast_precision_loss)]
        let bytes_per_second = bytes_per_second as f32;
        let frame_seconds = self.state.frame_duration().get::<second>();
        let bytes_per_frame = bytes_per_second * frame_seconds;
        #[expect(clippy::cast_sign_loss)]
        #[expect(clippy::cast_possible_truncation)]
        let bytes_per_frame = bytes_per_frame as u32;
        Information::new::<byte>(bytes_per_frame)
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
    /// NOTE: `width` and `height` are the client area's size; the window grows to fit its frame
    /// around it. If that wouldn't fit on the monitor, the client area shrinks instead.
    fn start_application(
        &mut self,
        state: &Win32State,
        loader: &mut ApplicationLoader,
        width: u16,
        height: u16,
    ) -> Result<()> {
        let LoadedApplication::Fresh(application) = loader.load(&mut self.plugin_state)? else {
            return Err(ApplicationError::new(
                "The plugin was already running before the game started",
            ));
        };

        self.create_window(state, &application.name(), width, height)?;

        self.monitors = find_monitors(self.window.handle());
        let frame_duration = self.find_frame_duration(&application);
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

    /// Runs at the start of the first frame after the window lands on another monitor or the
    /// monitors are reconfigured, such as the player going fullscreen on a faster monitor. The
    /// game gets to suggest a frame duration for the new monitors, and the audio written each
    /// frame follows it.
    fn handle_monitor_change(
        &mut self,
        application: &ApplicationStub,
        sound_output: Option<&mut Win32SoundOutput<'_>>,
    ) {
        self.monitors_changed = false;
        let frame_duration = self.find_frame_duration(application);
        if frame_duration == self.state.frame_duration() {
            return;
        }
        self.state.set_frame_duration(frame_duration);
        if let Some(sound_output) = sound_output {
            let format = self.state.audio().format();
            sound_output.set_frame_size(self.sample_size_per_frame(format));
        }
    }

    fn find_frame_duration(&self, application: &ApplicationStub) -> Time {
        let monitors = self.game_monitors();
        let duration = application.suggest_frame_duration(&monitors);
        duration.unwrap_or_else(Self::default_frame_duration)
    }

    fn default_frame_duration() -> Time {
        let refresh_rate = uom::si::f32::Frequency::new::<hertz>(60.0);
        2.0 / refresh_rate
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

    fn process_recording(&mut self, state: &Win32State, application: &ApplicationStub) {
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
            // Scrolling during playback is dropped, rather than arriving all at once when live
            // input resumes.
            state.mouse_mut().capture_wheel();
        } else {
            // The keyboard and mouse have been accumulating events all frame. Publish a copy as
            // the input the game sees. Because this happens every live frame, stopping playback
            // needs no special reset: the next frame simply shows the real keys again.
            *self.input.keyboard_mut() = state.keyboard().clone();
            self.poll_all_controller_state();
            self.capture_mouse_state(state);

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
        let monitors = &self.monitors;
        let list_monitors = || -> Vec<Monitor> {
            monitors
                .iter()
                .map(Win32Monitor::monitor)
                .cloned()
                .collect()
        };
        let context = InputContext {
            input_state: &self.input,
            game_state: &mut self.state,
            plugin_state: plugin,
            display: Display::new(&list_monitors, &mut self.display),
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

    fn capture_mouse_state(&mut self, state: &Win32State) {
        let mut mouse = state.mouse_mut();
        if let Ok(client_coordinate) = self.window.client_coordinate() {
            mouse
                .capture_position(client_coordinate)
                .unwrap_or_default(); // Ignore errors
        }
        mouse.capture_wheel();
        *self.input.mouse_mut() = mouse.state().clone();
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
        let bytes_to_write = Self::bytes_to_target(write_offset, target_cursor, buffer_length);
        // The target cursor is estimated from elapsed time, so it can land partway through a
        // sample. Rounding down keeps every write offset on a sample boundary, which the ring
        // buffer math cannot guarantee on its own since offsets are bytes. The stray bytes are
        // covered by the next frame's write.
        let sample_size = self.state.audio().format().sample_size().get::<byte>();
        let misaligned_bytes = bytes_to_write.checked_rem(sample_size).unwrap_or(0);
        let aligned_bytes_to_write = bytes_to_write.saturating_sub(misaligned_bytes);
        Information::new::<byte>(aligned_bytes_to_write)
    }

    /// Bytes to write to carry the ring buffer forward from `write_offset` to `target_cursor`.
    ///
    /// Earlier frames may already have written past the target. That happens when the frame
    /// duration shrinks by more than half, since the last write reached one old frame ahead.
    /// Read as a wrap around the ring, that would be a write of almost the whole buffer over
    /// audio about to play, so nothing is written instead. A write offset that fell behind the
    /// play cursor is the opposite case, lagging rather than leading, so it sits most of the ring
    /// past the target and still catches up.
    fn bytes_to_target(write_offset: u32, target_cursor: u32, buffer_length: u32) -> u32 {
        let lead_over_target = Self::ring_distance(target_cursor, write_offset, buffer_length);
        if lead_over_target < buffer_length / 2 {
            return 0;
        }
        Self::ring_distance(write_offset, target_cursor, buffer_length)
    }

    /// How many bytes forward it is from `from` to `to`, going around the ring buffer if needed.
    fn ring_distance(from: u32, to: u32, buffer_length: u32) -> u32 {
        if to >= from {
            to - from
        } else {
            buffer_length - from + to
        }
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

#[cfg(test)]
mod tests {
    use super::Win32Application;

    const BUFFER_LENGTH: u32 = 1000;

    #[test]
    fn test_bytes_to_target_writes_up_to_a_target_ahead() {
        assert_eq!(
            200,
            Win32Application::bytes_to_target(300, 500, BUFFER_LENGTH)
        );
    }

    #[test]
    fn test_bytes_to_target_wraps_around_the_end_of_the_buffer() {
        assert_eq!(
            150,
            Win32Application::bytes_to_target(900, 50, BUFFER_LENGTH)
        );
    }

    #[test]
    fn test_bytes_to_target_writes_nothing_when_already_past_the_target() {
        assert_eq!(
            0,
            Win32Application::bytes_to_target(500, 450, BUFFER_LENGTH)
        );
    }

    #[test]
    fn test_bytes_to_target_writes_nothing_when_past_a_target_across_the_wrap() {
        assert_eq!(0, Win32Application::bytes_to_target(20, 980, BUFFER_LENGTH));
    }

    #[test]
    fn test_bytes_to_target_catches_up_when_the_write_offset_lags_across_the_wrap() {
        // The play cursor has overtaken the write offset, so the target is far ahead of it.
        assert_eq!(
            450,
            Win32Application::bytes_to_target(850, 300, BUFFER_LENGTH)
        );
    }

    #[test]
    fn test_bytes_to_target_writes_nothing_at_the_target() {
        assert_eq!(
            0,
            Win32Application::bytes_to_target(400, 400, BUFFER_LENGTH)
        );
    }
}
