use bevy::prelude::*;

use crate::egui::SimControls;

pub fn handle_keystrokes(mut kbd: ResMut<ButtonInput<KeyCode>>, mut controls: ResMut<SimControls>) {
    if kbd.clear_just_pressed(KeyCode::KeyR) {
        controls.restart_requested = true;
    }
    if kbd.clear_just_pressed(KeyCode::Space) {
        controls.paused ^= true;
    }
    if kbd.clear_just_pressed(KeyCode::KeyS) {
        controls.single_step = true;
        controls.paused = false;
    }
}
