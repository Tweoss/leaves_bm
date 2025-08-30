use bevy::prelude::*;

use crate::egui::RestartSim;

pub fn handle_keystrokes(mut kbd: ResMut<ButtonInput<KeyCode>>, mut restart: ResMut<RestartSim>) {
    if kbd.clear_just_pressed(KeyCode::KeyR) {
        restart.0 = true;
    }
}
