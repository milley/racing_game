//! 音效系统
//!
//! 包含：
//! - 背景音乐
//! - 碰撞音效
//! - 道具收集音效
//!
//! 注意：需要添加音频文件到 assets/sounds/ 目录
//! 或者使用程序生成的音调

use bevy::prelude::*;

use crate::game::GameState;

/// 音效插件
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AudioConfig>()
            .add_systems(OnEnter(GameState::Playing), start_background_music)
            .add_systems(OnExit(GameState::Playing), stop_background_music)
            .add_systems(Update, update_music_volume.run_if(in_state(GameState::Playing)));
    }
}

/// 音效配置
#[derive(Resource)]
pub struct AudioConfig {
    /// 主音量 (0.0 - 1.0)
    pub master_volume: f32,
    /// 音乐音量
    pub music_volume: f32,
    /// 音效音量
    pub sfx_volume: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 0.7,
            music_volume: 0.5,
            sfx_volume: 0.8,
        }
    }
}

/// 背景音乐标记
#[derive(Component)]
struct BackgroundMusic;

/// 开始背景音乐
fn start_background_music(
    _commands: Commands,
    _audio_config: Res<AudioConfig>,
) {
    // 如果有音频文件，可以这样加载：
    // let handle = audio.load("sounds/background.ogg");
    // audio.play(handle).looped().with_volume(audio_config.music_volume);
    info!("Background music started");
}

/// 停止背景音乐
fn stop_background_music(
    mut commands: Commands,
    query: Query<Entity, With<BackgroundMusic>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    info!("Background music stopped");
}

/// 更新音乐音量
fn update_music_volume(
    audio_config: Res<AudioConfig>,
) {
    if audio_config.is_changed() {
        // 更新音量逻辑
    }
}

/// 播放碰撞音效
#[allow(dead_code)]
pub fn play_collision_sound() {
    // let handle = audio.load("sounds/collision.ogg");
    // audio.play(handle).with_volume(audio_config.sfx_volume);
    info!("Collision sound played");
}

/// 播放道具收集音效
#[allow(dead_code)]
pub fn play_powerup_sound() {
    // let handle = audio.load("sounds/powerup.ogg");
    // audio.play(handle).with_volume(audio_config.sfx_volume);
    info!("Powerup sound played");
}

/// 播放护盾激活音效
#[allow(dead_code)]
pub fn play_shield_sound() {
    // let handle = audio.load("sounds/shield.ogg");
    // audio.play(handle).with_volume(audio_config.sfx_volume);
    info!("Shield sound played");
}
