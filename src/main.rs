mod game_objects_plugin;
mod input_plugin;
mod mouse_tracking_plugin;
mod movement_plugin;
mod sound_plugin;

use bevy::prelude::*;

const WINDOW_SIZE: Vec2 = Vec2::new(800., 400.);
const PLAYER_ACCELERATION: f32 = 1024.;
const PLAYER_MAX_SPEED: f32 = 300.;

const PLAYER_SIZE: f32 = 4.;

#[derive(Resource)]
struct WorldBorders {
    width: u32,
    height: u32,
}

impl Default for WorldBorders {
    fn default() -> Self {
        WorldBorders {
            width: WINDOW_SIZE.x as u32,
            height: WINDOW_SIZE.y as u32,
        }
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set( WindowPlugin {
                primary_window: Some(
                    Window {
                        resolution: WINDOW_SIZE.into(),
                        resizable: false,
                        ..default()
                    }
                ),
                ..default()
            },)
            .set( ImagePlugin::default_nearest(),
            )
    );
    app.add_plugins((movement_plugin::MovementPlugin, input_plugin::InputPlugin, mouse_tracking_plugin::MouseTrackingPlugin, game_objects_plugin::GameObjectsPlugin, sound_plugin::SoundPlugin));
    app.add_systems(Update, (bevy::window::close_on_esc, move_player));
    app.add_systems(Startup, setup);
    app.init_resource::<WorldBorders>();
    app.run();
}

fn setup (
    mut commands: Commands,
) {

    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = bevy::render::camera::ScalingMode::Fixed {
        width: WINDOW_SIZE.x,
        height: WINDOW_SIZE.y
    };
    camera_bundle.camera.clear_color = ClearColorConfig::Custom(Color::rgb(32. / 255., 32. / 255., 64. / 255.));
    commands.spawn(camera_bundle);
}

fn move_player(
    mut sprite_query: Query<&mut movement_plugin::TranslationalPhysics, With<input_plugin::InputResponsive>>,
    mut ev_reader: EventReader<input_plugin::InputEvent>,
) {
    for mut physics in &mut sprite_query {
        let mut acceleration = Vec2::splat(0.);
        for ev in ev_reader.read() {
            acceleration += match ev {
                input_plugin::InputEvent::Up => Vec2::new(0., 1.),
                input_plugin::InputEvent::Down => Vec2::new(0., -1.),
                input_plugin::InputEvent::Left => Vec2::new(-1., 0.),
                input_plugin::InputEvent::Right => Vec2::new(1., 0.),
                _ => continue,
            }
        }
        physics.acceleration += acceleration.normalize_or_zero() * PLAYER_ACCELERATION;
    }
}
