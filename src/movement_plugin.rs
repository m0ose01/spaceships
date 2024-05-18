use bevy::{
    prelude::*,
};

use bevy_xpbd_2d::prelude::*;

use crate::game_objects_plugin::AutoCollider;

use crate::mouse_tracking_plugin::MouseWorldCoords;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate,
            (
                rotate_to_mouse,
                calculate_hitbox,
                wrap_sprite,
                collide_sound,
                limit_max_speed,
                collide_damage,
            ).chain()
        );
        app.insert_resource(Gravity(Vec2::ZERO));
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
    mut sprite_query: Query<(&mut LinearVelocity, &MaxSpeed)>,
) {
    for (mut velocity, max_speed) in &mut sprite_query {
        if velocity.0.length() > max_speed.speed {
            velocity.0 = velocity.normalize() * max_speed.speed;
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

fn calculate_hitbox(
    assets: Res<Assets<Image>>,
    sprite_query: Query<(Entity, &Handle<Image>, &AutoCollider)>,
    mut commands: Commands,
) {
    for (entity, sprite_handle, autocollider) in &sprite_query {
        let size = match assets.get(sprite_handle) {
            Some(vec) => vec.size_f32(),
            None => Vec2::splat(0.),
        };

        let hitbox_shrinking_factor = 0.9;
        let border_radius = 4.;

        let size_scaled = Vec2::new (
            size.x * hitbox_shrinking_factor,
            size.y * hitbox_shrinking_factor,
        );

        let collider = match autocollider {
            AutoCollider::Circle => Collider::circle(
                (size_scaled.x + size_scaled.y) / 4.,
            ),
            AutoCollider::RoundedRectangle => Collider::round_rectangle(
               size_scaled.x, size_scaled.y, border_radius,
            ),
        };

        if let Some(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.insert(collider);
            entity_commands.remove::<AutoCollider>();
        }
    }
}

fn collide_sound(
    mut event_reader: EventReader<Collision>,
    mut event_writer: EventWriter<crate::sound_plugin::SoundEffectEvent>,
) {
    for _ in event_reader.read() {
        event_writer.send(crate::sound_plugin::SoundEffectEvent::CollisionSound);
    }
}

fn collide_damage(
    sprite_query: Query<Entity>,
    mut ev_writer: EventWriter<crate::game_objects_plugin::DamageEvent>,
    mut ev_reader: EventReader<Collision>,
) {
    let damage = 20;
    for Collision(contacts) in ev_reader.read() {
        if sprite_query.get_many([contacts.entity1, contacts.entity2]).is_ok() {
            ev_writer.send(crate::game_objects_plugin::DamageEvent::new(damage, contacts.entity1));
            ev_writer.send(crate::game_objects_plugin::DamageEvent::new(damage, contacts.entity2));
        }
    }
}
