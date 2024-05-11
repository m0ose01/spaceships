use bevy::prelude::*;

const WINDOW_SIZE: Vec2 = Vec2::new(800., 400.);
const PLAYER_ACCELERATION: f32 = 20.;
const PLAYER_MAX_SPEED: f32 = 300.;

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
    app.add_plugins((movement_plugin::MovementPlugin, input_plugin::InputPlugin, mouse_tracking_plugin::MouseTrackingPlugin));
    app.add_systems(Update, (bevy::window::close_on_esc, move_player));
    app.add_systems(Startup, setup);
    app.run();
}

fn setup (
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {

    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = bevy::render::camera::ScalingMode::Fixed {
        width: WINDOW_SIZE.x,
        height: WINDOW_SIZE.y
    };
    commands.spawn(camera);
    commands.spawn((
        movement_plugin::Physics::default(),
        movement_plugin::MaxSpeed::new(PLAYER_MAX_SPEED),
        SpriteBundle {
            texture: asset_server.load("textures/spaceship.png"),
            ..default()
        },
        input_plugin::InputResponsive,
    ));
}

fn move_player(
    mut sprite_query: Query<&mut movement_plugin::Physics, With<input_plugin::InputResponsive>>,
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

mod movement_plugin {

    use bevy::prelude::*;

    pub struct MovementPlugin;

    impl Plugin for MovementPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Update, (
                update_physics,
                limit_max_speed,
                move_sprite,
            ).chain()
            );
        }
    }

    #[derive(Component)]
    pub struct Physics {
        pub velocity: Vec2,
        pub acceleration: Vec2,
    }

    impl Physics {
        pub fn update(&mut self) {
            self.velocity += self.acceleration;
            self.acceleration = Vec2::splat(0.);
        }
    }


    impl Default for Physics {
        fn default() -> Self {
            Self {
                velocity: Vec2::splat(0.),
                acceleration: Vec2::splat(0.),
            }
        }
    }

    fn update_physics(
        mut sprite_query: Query<&mut Physics>,
    ) {
        for mut physics in &mut sprite_query {
            physics.update();
        }
    }

    fn move_sprite(
        time: Res<Time>,
        mut sprite_query: Query<(&mut Transform, &Physics)>,
    ) {
        let deltat_s = time.delta_seconds();
        for (mut transform, physics) in &mut sprite_query {
            transform.translation.x += physics.velocity.x * deltat_s;
            transform.translation.y += physics.velocity.y * deltat_s;
        }
    }

    #[derive(Component)]
    pub struct MaxSpeed {
        speed: f32,
    }

    impl MaxSpeed {
        pub fn new(speed: f32) -> Self {
            MaxSpeed {
                speed
            }
        }
    }

    fn limit_max_speed(
        mut sprite_query: Query<(&mut Physics, &MaxSpeed)>,
    ) {
        for (mut physics, max_speed) in &mut sprite_query {
            if physics.velocity.length() > max_speed.speed {
                physics.velocity = physics.velocity.normalize() * max_speed.speed;
            }
        }
    }
}

mod input_plugin {

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

        if mouse.any_pressed(primary_action_keys) {
            ev_writer.send(InputEvent::PrimaryAction);
        }
        if mouse.any_pressed(secondary_action_keys) {
            ev_writer.send(InputEvent::SecondaryAction);
        }
    }
}

mod mouse_tracking_plugin {

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
    pub struct MouseWorldCoords(Vec2);

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
}
