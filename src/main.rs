mod game_objects_plugin;
mod input_plugin;
mod mouse_tracking_plugin;
mod movement_plugin;
mod sound_plugin;

use bevy::prelude::*;
use bevy_xpbd_2d::prelude::*;

const WINDOW_SIZE: Vec2 = Vec2::new(800., 400.);
const PLAYER_ACCELERATION: f32 = 16.;
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
    app.add_plugins(PhysicsPlugins::default());
    // app.add_plugins(PhysicsDebugPlugin::default());
    app.add_systems(Update, bevy::window::close_on_esc);
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
