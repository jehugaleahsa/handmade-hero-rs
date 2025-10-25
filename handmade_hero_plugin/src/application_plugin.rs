use handmade_hero_interface::application::Application;
use handmade_hero_interface::audio_context::AudioContext;
use handmade_hero_interface::button_state::ButtonState;
use handmade_hero_interface::controller_state::ControllerState;
use handmade_hero_interface::game_state::GameState;
use handmade_hero_interface::input_context::InputContext;
use handmade_hero_interface::input_state::InputState;
use handmade_hero_interface::pixel::Pixel;
use handmade_hero_interface::render_context::RenderContext;
use handmade_hero_interface::stereo_sample::StereoSample;
use std::ops::Neg;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ApplicationPlugin;

impl ApplicationPlugin {
    #[unsafe(no_mangle)]
    #[must_use]
    pub extern "Rust" fn create_application() -> Box<dyn Application> {
        Box::new(ApplicationPlugin)
    }

    fn handle_keyboard_input(input: &InputState, state: &mut GameState) {
        let keyboard = input.keyboard();
        if !keyboard.enabled() {
            return;
        }

        let shift_left = keyboard.left().half_transition_count();
        let shift_right = keyboard.right().half_transition_count();
        let shift_x = shift_right.cast_signed() - shift_left.cast_signed();

        let shift_up = keyboard.up().half_transition_count();
        let shift_down = keyboard.down().half_transition_count();
        let shift_y = shift_up.cast_signed() - shift_down.cast_signed();

        state.shift_x(shift_x.neg().saturating_mul(5));
        state.shift_y(shift_y.saturating_mul(5));
    }

    fn handle_controller_input(controller: &ControllerState, state: &mut GameState) {
        if !controller.enabled() {
            return;
        }
        Self::shift_x_using_controller(controller, state);
        Self::shift_y_using_controller(controller, state);
        Self::set_hertz_using_controller(controller, state);

        if controller.a().ended_down() && state.jump_time() == 0f32 {
            state.set_jump_time(1.0f32);
        } else if state.jump_time() > 0f32 {
            state.set_jump_time(f32::max(0f32, state.jump_time() - 0.033f32));
        }
    }

    fn shift_x_using_controller(controller: &ControllerState, state: &mut GameState) {
        let shift_x_ratio = if controller.left().ended_down() {
            1.0f32
        } else if controller.right().ended_down() {
            -1.0f32
        } else {
            -(controller.left_joystick().x() + controller.right_joystick().x())
        };
        #[allow(clippy::cast_possible_truncation)]
        let shift_x = (shift_x_ratio * 5f32) as i16;
        state.shift_x(shift_x.neg());
        state.set_player_x(
            state
                .player_x()
                .cast_signed()
                .wrapping_sub(shift_x)
                .unsigned_abs(),
        );
    }

    fn shift_y_using_controller(controller: &ControllerState, state: &mut GameState) {
        let shift_y_ratio = if controller.up().ended_down() {
            1.0f32
        } else if controller.down().ended_down() {
            -1.0f32
        } else {
            controller.left_joystick().y() + controller.right_joystick().y()
        };
        #[allow(clippy::cast_possible_truncation)]
        let shift_y = (shift_y_ratio * 5f32) as i16;
        state.shift_y(shift_y.neg());
        state.set_player_y(
            state
                .player_y()
                .cast_signed()
                .wrapping_sub(shift_y)
                .unsigned_abs(),
        );
    }

    fn set_hertz_using_controller(controller: &ControllerState, state: &mut GameState) {
        let left_thumb_y_ratio = if controller.up().ended_down() {
            1.0f32
        } else if controller.down().ended_down() {
            -1.0f32
        } else {
            controller.left_joystick().y()
        };
        let right_thumb_y_ratio = controller.right_joystick().y();
        let thumb_y_ratio = f32::midpoint(left_thumb_y_ratio, right_thumb_y_ratio);
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        let hertz = (512.0f32 + (256.0f32 * thumb_y_ratio)) as u32;
        state.set_sound_hertz(hertz);
    }

    fn render_player(state: &GameState, buffer: &mut [Pixel]) {
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        let player_top =
            (f32::from(state.player_y()) + -100f32 * (state.jump_time() * 2f32).sin()) as usize;
        let player_left = usize::from(state.player_x());
        let player_bottom = player_top.saturating_add(usize::from(GameState::PLAYER_HEIGHT));
        let player_right = player_left.saturating_add(usize::from(GameState::PLAYER_WIDTH));
        let pitch = usize::from(state.width());
        let pixel = Pixel::from_rgb(0xFF, 0xFF, 0x00);
        for y in player_top..player_bottom {
            for x in player_left..player_right {
                let index = y * pitch + x;
                buffer[index] = pixel;
            }
        }
    }

    fn render_mouse(input: &InputState, state: &GameState, buffer: &mut [Pixel]) {
        let mouse = input.mouse();
        let player_height = u32::from(GameState::PLAYER_HEIGHT);
        let mouse_top = u32::max(0, mouse.y().saturating_sub(player_height / 2));
        let mouse_bottom = mouse_top.saturating_add(player_height);
        let height = u32::from(state.height());
        let mouse_bottom = u32::min(mouse_bottom, height);

        let player_width = u32::from(GameState::PLAYER_WIDTH);
        let mouse_left = u32::max(0, mouse.x().saturating_sub(player_width / 2));
        let mouse_right = mouse_left.saturating_add(player_width);
        let width = u32::from(state.width());
        let mouse_right = u32::min(mouse_right, width);

        let red = Self::intensity(*mouse.left());
        let green = Self::intensity(*mouse.middle());
        let blue = Self::intensity(*mouse.right());
        let pixel = Pixel::from_rgb(red, green, blue);
        for y in mouse_top..mouse_bottom {
            for x in mouse_left..mouse_right {
                let index = y * width + x;
                let index = index as usize;
                buffer[index] = pixel;
            }
        }
    }

    fn intensity(button: ButtonState) -> u8 {
        if button.ended_down() { 0xFF } else { 0x00 }
    }
}

impl Application for ApplicationPlugin {
    fn process_input(&self, context: InputContext<'_>) {
        let InputContext { input, state } = context;
        Self::handle_keyboard_input(input, state);
        for controller in input.controllers() {
            Self::handle_controller_input(controller, state);
        }
    }

    fn render(&self, context: RenderContext<'_>) {
        let RenderContext {
            input,
            state,
            buffer,
        } = context;
        let height = state.height();
        let width = state.width();
        let x_offset = state.x_offset();
        let y_offset = state.y_offset();

        let mut index = 0;
        for y in 0..height {
            for x in 0..width {
                let color = Pixel::from_rgb(
                    0x00,
                    (y.wrapping_add(y_offset) & 0xFF) as u8,
                    (x.wrapping_add(x_offset) & 0xFF) as u8,
                );
                buffer[index] = color;
                index += 1;
            }
        }
        Self::render_player(state, buffer);
        Self::render_mouse(input, state, buffer);
    }

    fn write_sound(&self, context: AudioContext<'_>) {
        let AudioContext {
            state,
            sound_buffer,
        } = context;
        let time_delta = state.time_delta();
        let volume = f32::from(state.sound_volume());
        for sample in sound_buffer {
            let sine_value = state.sound_theta().sin();
            #[allow(clippy::cast_possible_truncation)]
            let sample_value = (sine_value * volume) as i16;
            *sample = StereoSample::from_left_right(sample_value, sample_value);
            state.advance_sound_theta(time_delta);
        }
    }
}
