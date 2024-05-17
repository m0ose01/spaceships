use bevy::{
    prelude::*,
    math::bounding,
    math::bounding::IntersectsVolume,
};

use crate::mouse_tracking_plugin::MouseWorldCoords;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CollisionEvent>();
        app.add_systems(FixedUpdate,
            (
                accelerate_sprite_rotation,
                rotate_to_mouse,
                rotate_sprite,
                wrap_sprite,
                calculate_hitbox,
                check_collisions,
                collide,
                accelerate_sprite,
                limit_max_speed,
                translate_sprite,
                collide_damage,
            ).chain()
        );
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
            Some(sprite_handle) => match assets.get(sprite_handle) {
                Some(vec) => vec.size(),
                None => UVec2::splat(0),
            },
            None => UVec2::splat(0),
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

#[derive(Component)]
pub struct CollisionPhysics {
    hitbox: bounding::BoundingCircle,
}

impl Default for CollisionPhysics {
    fn default() -> Self {
        CollisionPhysics {
            hitbox: bounding::BoundingCircle::new(
                Vec2::splat(0.),
                0.,
            ),
        }
    }
}

fn calculate_hitbox(
    assets: Res<Assets<Image>>,
    mut sprite_query: Query<(&mut CollisionPhysics, &Transform, &Handle<Image>)>,
) {
    for (mut collision_physics, transform, sprite_handle) in &mut sprite_query {
        let size = match assets.get(sprite_handle) {
            Some(vec) => vec.size_f32(),
            None => Vec2::splat(0.),
        };

        let hitbox_shrinking_factor = 0.9;

        let size_scaled = Vec2::new (
            size.x * transform.scale.x * hitbox_shrinking_factor,
            size.y * transform.scale.y * hitbox_shrinking_factor,
        );

        let bounding_circle = bounding::BoundingCircle::new(
            transform.translation.truncate(),
            (size_scaled.x + size_scaled.y) / 4.,
        );
        collision_physics.hitbox = bounding_circle;
    }
}

#[derive(Event)]
struct CollisionEvent {
    entity_1: Entity,
    entity_2: Entity,
}

fn check_collisions(
    mut event_writer: EventWriter<CollisionEvent>,
    sprite_query: Query<(&CollisionPhysics, Entity)>,
) {
    let mut combinations = sprite_query.iter_combinations();
    while let Some([(collision_physics_1, entity_1), (collision_physics_2, entity_2)]) = combinations.fetch_next() {
        if collision_physics_1.hitbox.intersects(&collision_physics_2.hitbox) {
            event_writer.send(CollisionEvent {
                entity_1,
                entity_2,
            });
        }
    }
}

fn collide(
    mut sprite_query: Query<(&mut TranslationalPhysics, &CollisionPhysics)>,
    mut event_reader: EventReader<CollisionEvent>,
    mut event_writer: EventWriter<crate::sound_plugin::SoundEffectEvent>,
) {
    for collision_event in event_reader.read() {
        if let Ok(mut query) = sprite_query.get_many_mut([collision_event.entity_1, collision_event.entity_2]) {
            let velocity_a = query[0].0.velocity;
            let velocity_b = query[1].0.velocity;
            let hitbox_a = query[0].1.hitbox;
            let hitbox_b = query[1].1.hitbox;
            let vector_between = hitbox_a.center - hitbox_b.center;
            let rebound = 1.5;
            let energy_loss = 0.5;
            query[0].0.velocity = velocity_b * energy_loss + vector_between * rebound;
            query[1].0.velocity = velocity_a * energy_loss - vector_between * rebound;
            event_writer.send(crate::sound_plugin::SoundEffectEvent::CollisionSound);
        }
    }
}

fn collide_damage(
    sprite_query: Query<Entity>,
    mut ev_writer: EventWriter<crate::game_objects_plugin::DamageEvent>,
    mut ev_reader: EventReader<CollisionEvent>,
) {
    let damage = 20;
    for ev in ev_reader.read() {
        if sprite_query.get_many([ev.entity_1, ev.entity_2]).is_ok() {
            ev_writer.send(crate::game_objects_plugin::DamageEvent::new(damage, ev.entity_1));
            ev_writer.send(crate::game_objects_plugin::DamageEvent::new(damage, ev.entity_2));
        }
    }
}