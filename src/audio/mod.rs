//! 音效系统
//!
//! 包含：
//! - 背景音乐
//! - 碰撞音效
//! - 道具收集音效
//!
//! 注意：需要添加音频文件到 assets/sounds/ 目录：
//! - background.ogg (背景音乐)
//! - collision.ogg (碰撞音效)
//! - powerup.ogg (道具收集音效)
//! - shield.ogg (护盾激活音效)

use bevy::prelude::*;
use bevy::audio::Volume;

use crate::game::GameState;
use crate::settings::GameSettings;

/// 音效插件
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AudioAssets>()
            .add_systems(OnEnter(GameState::Playing), start_background_music)
            .add_systems(OnExit(GameState::Playing), stop_background_music)
            .add_systems(Update, update_music_volume.run_if(in_state(GameState::Playing)));
    }
}

/// 音频资源
#[derive(Resource, Default)]
pub struct AudioAssets {
    /// 背景音乐
    background: Option<Handle<AudioSource>>,
    /// 碰撞音效
    collision: Option<Handle<AudioSource>>,
    /// 道具收集音效
    powerup: Option<Handle<AudioSource>>,
    /// 护盾激活音效
    shield: Option<Handle<AudioSource>>,
}

/// 背景音乐实体标记
#[derive(Component)]
struct BackgroundMusicEntity;

/// 开始背景音乐
fn start_background_music(
    mut commands: Commands,
    mut audio_assets: ResMut<AudioAssets>,
    settings: Res<GameSettings>,
    asset_server: Res<AssetServer>,
) {
    // 加载音频资源
    if audio_assets.background.is_none() {
        audio_assets.background = Some(asset_server.load("sounds/background.ogg"));
        audio_assets.collision = Some(asset_server.load("sounds/collision.ogg"));
        audio_assets.powerup = Some(asset_server.load("sounds/powerup.ogg"));
        audio_assets.shield = Some(asset_server.load("sounds/shield.ogg"));
    }

    // 播放背景音乐
    if let Some(handle) = &audio_assets.background {
        let volume = Volume::Linear(settings.master_volume * settings.music_volume);
        commands.spawn((
            AudioPlayer(handle.clone()),
            PlaybackSettings::LOOP.with_volume(volume),
            BackgroundMusicEntity,
        ));
        info!("Background music started with volume {:?}", volume);
    } else {
        info!("Background music started (no audio file)");
    }
}

/// 停止背景音乐
fn stop_background_music(
    mut commands: Commands,
    query: Query<Entity, With<BackgroundMusicEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    info!("Background music stopped");
}

/// 更新音乐音量
fn update_music_volume(
    settings: Res<GameSettings>,
    mut query: Query<&mut PlaybackSettings, With<BackgroundMusicEntity>>,
) {
    if settings.is_changed() {
        let volume = Volume::Linear(settings.master_volume * settings.music_volume);
        for mut playback in query.iter_mut() {
            playback.volume = volume;
        }
        info!("Music volume updated to {:?}", volume);
    }
}

/// 播放碰撞音效
pub fn play_collision_sound(
    commands: &mut Commands,
    audio_assets: &AudioAssets,
    settings: &GameSettings,
) {
    if let Some(handle) = &audio_assets.collision {
        let volume = Volume::Linear(settings.master_volume * settings.sfx_volume);
        commands.spawn((
            AudioPlayer(handle.clone()),
            PlaybackSettings::DESPAWN.with_volume(volume),
        ));
        info!("Collision sound played with volume {:?}", volume);
    }
}

/// 播放道具收集音效
pub fn play_powerup_sound(
    commands: &mut Commands,
    audio_assets: &AudioAssets,
    settings: &GameSettings,
) {
    if let Some(handle) = &audio_assets.powerup {
        let volume = Volume::Linear(settings.master_volume * settings.sfx_volume);
        commands.spawn((
            AudioPlayer(handle.clone()),
            PlaybackSettings::DESPAWN.with_volume(volume),
        ));
        info!("Powerup sound played with volume {:?}", volume);
    }
}

/// 播放护盾激活音效
pub fn play_shield_sound(
    commands: &mut Commands,
    audio_assets: &AudioAssets,
    settings: &GameSettings,
) {
    if let Some(handle) = &audio_assets.shield {
        let volume = Volume::Linear(settings.master_volume * settings.sfx_volume);
        commands.spawn((
            AudioPlayer(handle.clone()),
            PlaybackSettings::DESPAWN.with_volume(volume),
        ));
        info!("Shield sound played with volume {:?}", volume);
    }
}
