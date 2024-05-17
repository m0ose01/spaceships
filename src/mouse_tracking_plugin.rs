use bevy::{
    prelude::*,
    window::PrimaryWindow,
};

pub struct MouseTrackingPlugin;

impl Plugin for MouseTrackingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, cursor_tracking_system);
        app.init_resource::<MouseWorldCoords>();
    }
}

#[derive(Resource, Default)]
pub struct MouseWorldCoords(pub Vec2);

// From Bevy Unofficial Guide
fn cursor_tracking_system(
    mut mouse_coords: ResMut<MouseWorldCoords>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        mouse_coords.0 = world_position;
    }
    // println!("{:?}", mycoords.0)
}
