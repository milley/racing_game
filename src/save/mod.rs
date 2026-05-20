//! 存档系统
//!
//! 使用 JSON 文件持久化游戏数据

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

/// 存档插件
pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SaveData>()
            .add_systems(Startup, (load_save_data, apply_settings_from_save).chain());
    }
}

/// 存档数据
#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct SaveData {
    /// 最高分
    pub high_score: u32,
    /// 游戏统计
    pub stats: GameStats,
    /// 已解锁成就ID列表
    pub achievements: Vec<String>,
    /// 设置数据
    pub settings: SettingsData,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            high_score: 0,
            stats: GameStats::default(),
            achievements: Vec::new(),
            settings: SettingsData::default(),
        }
    }
}

/// 游戏统计数据
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct GameStats {
    /// 总游戏次数
    pub total_games_played: u32,
    /// 总得分
    pub total_score: u64,
    /// 最高达到的难度等级
    pub max_difficulty_reached: u32,
    /// 收集的道具数量
    pub powerups_collected: u32,
    /// 躲避的障碍物数量
    pub obstacles_dodged: u32,
}

/// 设置数据
#[derive(Serialize, Deserialize, Clone)]
pub struct SettingsData {
    /// 难度级别 (easy/normal/hard)
    pub difficulty: String,
    /// 主音量
    pub master_volume: f32,
    /// 音乐音量
    pub music_volume: f32,
    /// 音效音量
    pub sfx_volume: f32,
}

impl Default for SettingsData {
    fn default() -> Self {
        Self {
            difficulty: "normal".to_string(),
            master_volume: 0.8,
            music_volume: 0.5,
            sfx_volume: 0.7,
        }
    }
}

/// 存档文件路径
fn get_save_path() -> PathBuf {
    // 使用当前目录下的 save.json
    PathBuf::from("save.json")
}

/// 加载存档数据
fn load_save_data(mut save_data: ResMut<SaveData>) {
    let path = get_save_path();

    if path.exists() {
        match fs::File::open(&path) {
            Ok(mut file) => {
                let mut contents = String::new();
                if file.read_to_string(&mut contents).is_ok() {
                    match serde_json::from_str::<SaveData>(&contents) {
                        Ok(data) => {
                            *save_data = data;
                            info!("存档加载成功: {:?}", path);
                        }
                        Err(e) => {
                            warn!("存档解析失败，使用默认数据: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("无法打开存档文件: {}", e);
            }
        }
    }
}

/// 从存档应用设置
fn apply_settings_from_save(
    save_data: Res<SaveData>,
    mut settings: ResMut<crate::settings::GameSettings>,
) {
    settings.difficulty = crate::settings::DifficultyLevel::from_str(&save_data.settings.difficulty);
    settings.master_volume = save_data.settings.master_volume;
    settings.music_volume = save_data.settings.music_volume;
    settings.sfx_volume = save_data.settings.sfx_volume;
    info!("设置已从存档加载: difficulty={}", settings.difficulty.name());
}

/// 保存存档数据（在游戏结束时调用）
pub fn save_game_data(
    mut save_data: ResMut<SaveData>,
    score: Res<crate::game::Score>,
    game_timer: Res<crate::game::GameTimer>,
) {
    // 更新最高分
    if score.value > save_data.high_score {
        save_data.high_score = score.value;
    }

    // 更新统计
    save_data.stats.total_games_played += 1;
    save_data.stats.total_score += score.value as u64;

    // 更新最高难度
    let difficulty_level = (game_timer.elapsed / 10.0) as u32 + 1;
    if difficulty_level > save_data.stats.max_difficulty_reached {
        save_data.stats.max_difficulty_reached = difficulty_level;
    }

    // 保存到文件
    match save_to_file(&save_data) {
        Ok(_) => info!("存档保存成功"),
        Err(e) => error!("存档保存失败: {}", e),
    }
}

/// 保存数据到文件
pub fn save_to_file(data: &SaveData) -> io::Result<()> {
    let path = get_save_path();
    let json = serde_json::to_string_pretty(data)?;
    let mut file = fs::File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}
