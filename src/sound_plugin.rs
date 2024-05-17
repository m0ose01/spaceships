use bevy::prelude::*;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SoundEffectEvent>();
        app.add_systems(Update, play_sound_effect);
    }
}

#[derive(Event)]
pub enum SoundEffectEvent {
    CollisionSound,
    ShootBulletSound,
}

fn play_sound_effect(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut event_reader: EventReader<SoundEffectEvent>,
) {
    for event in event_reader.read() {
        let audio_handle: Handle<AudioSource> = match event {
            SoundEffectEvent::CollisionSound => asset_server.load("soundfx/RockImpact3.mp3"),
            SoundEffectEvent::ShootBulletSound => asset_server.load("soundfx/BulletWhoosh.mp3")
        };
        commands.spawn(
            AudioBundle {
                source: audio_handle,
                ..default()
            },
        );
    }
}
