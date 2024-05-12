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
    app.add_plugins((movement_plugin::MovementPlugin, input_plugin::InputPlugin, mouse_tracking_plugin::MouseTrackingPlugin, game_objects_plugin::GameObjectsPlugin));
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

mod movement_plugin {

    use bevy::prelude::*;

    use crate::mouse_tracking_plugin::MouseWorldCoords;

    pub struct MovementPlugin;

    impl Plugin for MovementPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Update, (
                accelerate_sprite,
                limit_max_speed,
                translate_sprite,
                rotate_sprite,
                accelerate_sprite_rotation,
            ).chain()
            );
            app.add_systems(Update, rotate_to_mouse);
            app.add_systems(Update, wrap_sprite);
        }
    }

    #[derive(Component)]
    pub struct TranslationalPhysics {
        pub velocity: Vec2,
        pub acceleration: Vec2,
    }

    impl Default for TranslationalPhysics {
        fn default() -> Self {
            Self {
                velocity: Vec2::splat(0.),
                acceleration: Vec2::splat(0.),
            }
        }
    }

    fn accelerate_sprite(
        time: Res<Time>,
        mut sprite_query: Query<&mut TranslationalPhysics>,
    ) {
        for mut physics in &mut sprite_query {
            physics.velocity = physics.velocity + physics.acceleration * time.delta_seconds();
            physics.acceleration = Vec2::splat(0.);
        }
    }

    fn translate_sprite(
        time: Res<Time>,
        mut sprite_query: Query<(&mut Transform, &TranslationalPhysics)>,
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
        mut sprite_query: Query<(&mut TranslationalPhysics, &MaxSpeed)>,
    ) {
        for (mut physics, max_speed) in &mut sprite_query {
            if physics.velocity.length() > max_speed.speed {
                physics.velocity = physics.velocity.normalize() * max_speed.speed;
            }
        }
    }

    #[derive(Component)]
    pub struct RotateToMouse;

    fn rotate_to_mouse(
        mouse_coords: Res<MouseWorldCoords>,
        mut sprite_query: Query<&mut Transform, With<RotateToMouse>>,
    ) {
        let mouse_position = mouse_coords.0;

        for mut transform in &mut sprite_query {
            let player_position = transform.translation.truncate();
            let vector_to_mouse = mouse_position - player_position;
            let angle_to_mouse = vector_to_mouse.y.atan2(vector_to_mouse.x);
            transform.rotation = Quat::from_rotation_z(angle_to_mouse - std::f32::consts::PI / 2.);
        }
    }

    #[derive(Component)]
    pub struct Wrap;

    fn wrap_sprite(
        assets: Res<Assets<Image>>,
        mut sprite_query: Query<(&mut Transform, Option<&Handle<Image>>), With<Wrap>>,
        world_borders: Res<crate::WorldBorders>,
    ) {
        for (mut transform, handle) in &mut sprite_query {
            let size = match handle {
                Some(sprite_handle) => assets.get(sprite_handle).expect("Could not get asset for a sprite handle.").size(),
                _ => UVec2::splat(0),
            };

            let size_scaled = Vec2::new (
                size.x as f32 * transform.scale.x,
                size.y as f32 * transform.scale.y,
            );

            let limits = Vec2::new (
                (world_borders.width as f32 + size_scaled.x) / 2.,
                (world_borders.height as f32 + size_scaled.y) / 2.,
            );

            if transform.translation.x.abs() > limits.x {
                transform.translation.x *= -1.;
            }
            if transform.translation.y.abs() > limits.y {
                transform.translation.y *= -1.;
            }
        }
    }

    #[derive(Component, Default)]
    pub struct RotationalPhysics {
        pub angular_velocity: f32,
        pub angular_acceleration: f32,
    }

    fn rotate_sprite(
        time: Res<Time>,
        mut sprite_query: Query<(&mut Transform, &RotationalPhysics)>,
    ) {
        for (mut transform, rotational_physics) in &mut sprite_query {
            transform.rotate_z(rotational_physics.angular_velocity * time.delta_seconds());
        }
    }

    fn accelerate_sprite_rotation(
        time: Res<Time>,
        mut sprite_query: Query<&mut RotationalPhysics>,
    ) {
        for mut rotational_physics in &mut sprite_query {
            rotational_physics.angular_velocity += rotational_physics.angular_acceleration * time.delta_seconds();
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
}

mod game_objects_plugin {

    use bevy::prelude::*;
    use rand::Rng;

    pub struct GameObjectsPlugin;

    impl Plugin for GameObjectsPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Startup, spawn_player);
            app.add_systems(Startup, spawn_asteroids);
        }
    }

    fn spawn_player (
        asset_server: Res<AssetServer>,
        mut commands: Commands,
    ) {

        let player = (
            crate::movement_plugin::TranslationalPhysics::default(),
            crate::movement_plugin::RotateToMouse,
            crate::movement_plugin::MaxSpeed::new(crate::PLAYER_MAX_SPEED),
            crate::movement_plugin::Wrap,
            SpriteBundle {
                texture: asset_server.load("textures/Spaceship.png"),
                transform: Transform::default().with_scale(Vec2::splat(crate::PLAYER_SIZE).extend(0.)),
                ..default()
            },
            crate::input_plugin::InputResponsive,
        );

        commands.spawn(
            player,
        );

    }

    fn spawn_asteroids(
        asset_server: Res<AssetServer>,
        mut commands: Commands,
        world_borders: Res<crate::WorldBorders>,
    ) {
        let asteroid_count = 5;
        let asteroid_speed = 256.;

        for _ in 0..asteroid_count {
            let asteroid = (
                crate::movement_plugin::TranslationalPhysics {
                    velocity: random_vector(asteroid_speed),
                    ..default()
                },
                SpriteBundle {
                    texture: asset_server.load("textures/Asteroid.png"),
                    transform: Transform::from_translation(
                        random_point(world_borders.width, world_borders.height),
                    ).with_scale(Vec2::splat(2.).extend(0.)),
                    ..default()
                },
                crate::movement_plugin::Wrap,
                crate::movement_plugin::RotationalPhysics {
                    angular_velocity: std::f32::consts::PI / 2.,
                    ..default()
                }
            );
            commands.spawn(asteroid);
        }
    }

    fn random_vector(speed: f32) -> Vec2 {
        let mut rng = rand::thread_rng();

        let rand1 = rng.gen::<f32>() - 0.5;
        let rand2 = rng.gen::<f32>() - 0.5;
        return Vec2::new(rand1, rand2).normalize_or_zero() * speed;
    }

    fn random_point(width: u32, height: u32) -> Vec3 {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(-(width as i32 / 2)..(width as i32 / 2)) as f32;
        let y = rng.gen_range(-(height as i32 / 2)..(height as i32 / 2)) as f32;
        Vec3::new(
            x,
            y,
            0.,
        )
    }
}
