use bevy::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, pc_input);
        app.add_event::<InputEvent>();
    }
}

#[derive(Event)]
pub enum InputEvent {
    Up,
    Down,
    Left,
    Right,
    PrimaryAction,
    SecondaryAction,
}

#[derive(Component)]
pub struct InputResponsive;

fn pc_input (
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut ev_writer: EventWriter<InputEvent>,
) {

    let up_keys = [KeyCode::KeyW, KeyCode::ArrowUp];
    let down_keys = [KeyCode::KeyS, KeyCode::ArrowDown];
    let left_keys = [KeyCode::KeyA, KeyCode::ArrowLeft];
    let right_keys = [KeyCode::KeyD, KeyCode::ArrowRight];

    if keys.any_pressed(up_keys) {
        ev_writer.send(InputEvent::Up);
    }
    if keys.any_pressed(down_keys) {
        ev_writer.send(InputEvent::Down);
    }
    if keys.any_pressed(left_keys) {
        ev_writer.send(InputEvent::Left);
    }
    if keys.any_pressed(right_keys) {
        ev_writer.send(InputEvent::Right);
    }

    let primary_action_keys = [MouseButton::Left];
    let secondary_action_keys = [MouseButton::Right];

    if mouse.any_just_pressed(primary_action_keys) {
        ev_writer.send(InputEvent::PrimaryAction);
    }
    if mouse.any_just_pressed(secondary_action_keys) {
        ev_writer.send(InputEvent::SecondaryAction);
    }

}
