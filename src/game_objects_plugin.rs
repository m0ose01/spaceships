use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
};
use rand::Rng;
use bevy_xpbd_2d::prelude::*;

pub struct GameObjectsPlugin;

const PLAYER_COLLISION_GROUP: u8 = 0;

impl Plugin for GameObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player);
        app.add_systems(Startup, spawn_asteroids);
        app.add_systems(Update, (
            shoot_bullet,
            deal_damage,
            kill,
            draw_health_bar,
        ));
        app.add_systems(PostProcessCollisions, (
            ignore_collisions,
        ));
        app.add_event::<DamageEvent>();
    }
}

#[derive(Component, PartialEq)]
pub struct CollisionGroup(u8);

fn spawn_player (
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {

    let player = (
        CollisionGroup(PLAYER_COLLISION_GROUP),
        Health::new(1000, 1000),
        ShowHealthBar,
        AutoCollider::Circle,
        RigidBody::Kinematic,
        crate::movement_plugin::RotateToMouse,
        crate::movement_plugin::MaxSpeed::new(crate::PLAYER_MAX_SPEED),
        crate::movement_plugin::Wrap,
        SpriteBundle {
            texture: asset_server.load("textures/Spaceship3.png"),
            transform: Transform::default().with_scale(Vec2::splat(crate::PLAYER_SIZE).extend(1.)),
            ..default()
        },
        crate::input_plugin::InputResponsive,
    );

    commands.spawn(
        player,
    );

}

#[derive(Component)]
pub enum AutoCollider {
    Circle,
    RoundedRectangle,
}

fn spawn_asteroids(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    world_borders: Res<crate::WorldBorders>,
) {
    let asteroid_count = 5;
    let asteroid_speed = 128.;
    let asteroid_restitution = 0.8;

    for _ in 0..asteroid_count {
        let asteroid = (
            AutoCollider::Circle,
            RigidBody::Dynamic,
            Health::new(500, 500),
            ShowHealthBar,
            LinearVelocity(random_vector(asteroid_speed)),
            crate::movement_plugin::MaxSpeed::new(asteroid_speed),
            Restitution::new(asteroid_restitution),
            SpriteBundle {
                texture: asset_server.load("textures/Asteroid2.png"),
                transform: Transform::from_translation(
                    random_point(world_borders.width, world_borders.height),
                ).with_scale(Vec2::splat(crate::PLAYER_SIZE).extend(1.)),
                ..default()
            },
            crate::movement_plugin::Wrap,
            AngularVelocity(std::f32::consts::PI / 2.),
        );
        commands.spawn(asteroid);
    }
}

fn shoot_bullet(
    asset_server: Res<AssetServer>,
    mouse_world_coords: Res<crate::mouse_tracking_plugin::MouseWorldCoords>,
    mut commands: Commands,
    mut ev_reader: EventReader<crate::input_plugin::InputEvent>,
    mut ev_writer: EventWriter<crate::sound_plugin::SoundEffectEvent>,
    sprite_query: Query<&Transform, With<crate::input_plugin::InputResponsive>>,
) {
    let player_position = match sprite_query.get_single() {
        Ok(transform) => transform.translation,
        Err(_) => return,
    };

    let mouse_vector = (mouse_world_coords.0.extend(0.) - player_position).normalize_or_zero();
    let bullet_size = 2.;
    for ev in ev_reader.read() {
        let bullet = (
            CollisionGroup(PLAYER_COLLISION_GROUP),
            Health::new(1, 1),
            SpriteBundle {
                transform: Transform::from_translation(player_position + mouse_vector * 32.).with_scale(Vec2::splat(bullet_size).extend(1.)),
                texture: asset_server.load("textures/Bullet.png"),
                ..default()
            },
            RigidBody::Kinematic,
            AutoCollider::Circle,
            LinearVelocity(mouse_vector.truncate() * 512.)
        );
        if let crate::input_plugin::InputEvent::PrimaryAction = ev {
            commands.spawn(bullet);
            ev_writer.send(crate::sound_plugin::SoundEffectEvent::ShootBulletSound);
        }
    }
}

#[derive(Component)]
pub struct Health {
    current: u32,
    max: u32,
}

impl Health {
    pub fn new(current: u32, max: u32) -> Self {
        Self {
            current,
            max,
        }
    }

    pub fn subtract(&mut self, amount: u32) -> u32 {
        let new_health = self.current as i32 - amount as i32;
        if new_health < 0 {
            self.current = 0;
        } else {
            self.current = new_health as u32;
        }
        self.current
    }

    pub fn add(&mut self, amount: u32) -> u32 {
        let new_health = self.current + amount;
        if new_health > self.max {
            self.current = self.max;
        } else {
            self.current = new_health;
        }
        self.current
    }
}

#[derive(Event)]
pub struct DamageEvent {
    damage: u32,
    entity: Entity,
}

impl DamageEvent {
    pub fn new(damage: u32, entity: Entity) -> Self {
        Self {
            damage,
            entity,
        }
    }

}

fn deal_damage(
    mut ev_reader: EventReader<DamageEvent>,
    mut sprite_query: Query<&mut Health>,
) {
    for ev in ev_reader.read() {
        if let Ok(mut health) = sprite_query.get_mut(ev.entity) {
            health.subtract(ev.damage);
        }
    }
}

fn kill (
    sprite_query: Query<(Entity, &Health)>,
    mut commands: Commands,
) {
    for (entity, health) in &sprite_query {
        if health.current == 0 {
            commands.entity(entity).despawn_recursive();
        }
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

fn ignore_collisions(
    mut collisions: ResMut<Collisions>,
    group_query: Query<(Entity, &CollisionGroup)>,
) {
    let mut collision_pairs = group_query.iter_combinations();
    while let Some([(entity_a, collision_group_a), (entity_b, collision_group_b)]) = collision_pairs.fetch_next() {
        if collision_group_a == collision_group_b {
            collisions.remove_collision_pair(entity_a, entity_b);
        }
    }
}

#[derive(Component)]
pub struct ShowHealthBar;

fn draw_health_bar(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    sprite_query: Query<(Entity, &Health), (With<ShowHealthBar>, Changed<Health>)>,
) {
    let health_bar_size = Vec2::new(8., 1.);
    let health_bar_vertical_offset = -4.;
    for (entity, health) in &sprite_query {
        if health.current == 0 {
            return;
        }
        let current_health_bar_width = (health.current as f32 / health.max as f32) * health_bar_size.x;
        let current_health_bar_size = Vec2::new(current_health_bar_width, health_bar_size.y);
        let current_health_bar_x_offset = -1. * (health_bar_size.x - current_health_bar_width) / 2.;
        let red_bar = MaterialMesh2dBundle {
            mesh: meshes.add(Rectangle::from_size(health_bar_size)).into(),
            transform: Transform::from_xyz(0., health_bar_vertical_offset, 1.),
            material: materials.add(Color::MAROON),
            ..default()
        };
        let green_bar = MaterialMesh2dBundle {
            mesh: meshes.add(Rectangle::from_size(current_health_bar_size)).into(),
            transform: Transform::from_xyz(current_health_bar_x_offset, health_bar_vertical_offset, 2.),
            material: materials.add(Color::DARK_GREEN),
            ..default()
        };
        if let Some(mut parent) = commands.get_entity(entity) {
            parent.despawn_descendants();
            commands.spawn(green_bar).set_parent(entity);
            commands.spawn(red_bar).set_parent(entity);
        }
    }
}
