//! 设置系统
//!
//! 包含难度选择和音量控制

use bevy::prelude::*;

use crate::game::GameState;
use crate::save::{SaveData, SettingsData};

/// 设置插件
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<GameSettings>()
            .add_systems(OnEnter(GameState::Settings), setup_settings)
            .add_systems(OnExit(GameState::Settings), cleanup_settings)
            .add_systems(Update, settings_system.run_if(in_state(GameState::Settings)));
    }
}

/// 难度级别
#[derive(Resource, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum DifficultyLevel {
    Easy,
    #[default]
    Normal,
    Hard,
}

impl DifficultyLevel {
    /// 获取难度名称
    pub fn name(&self) -> &'static str {
        match self {
            DifficultyLevel::Easy => "Easy",
            DifficultyLevel::Normal => "Normal",
            DifficultyLevel::Hard => "Hard",
        }
    }

    /// 获取速度倍率
    pub fn speed_multiplier(&self) -> f32 {
        match self {
            DifficultyLevel::Easy => 0.75,
            DifficultyLevel::Normal => 1.0,
            DifficultyLevel::Hard => 1.25,
        }
    }

    /// 获取生命值
    pub fn lives(&self) -> u32 {
        match self {
            DifficultyLevel::Easy => 5,
            DifficultyLevel::Normal => 3,
            DifficultyLevel::Hard => 2,
        }
    }

    /// 获取生成间隔倍率
    pub fn spawn_interval_multiplier(&self) -> f32 {
        match self {
            DifficultyLevel::Easy => 1.5,
            DifficultyLevel::Normal => 1.0,
            DifficultyLevel::Hard => 0.75,
        }
    }

    /// 从字符串解析
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "easy" => DifficultyLevel::Easy,
            "hard" => DifficultyLevel::Hard,
            _ => DifficultyLevel::Normal,
        }
    }

    /// 转换为字符串
    pub fn to_string(&self) -> String {
        self.name().to_lowercase()
    }
}

/// 游戏设置
#[derive(Resource)]
pub struct GameSettings {
    /// 难度级别
    pub difficulty: DifficultyLevel,
    /// 主音量
    pub master_volume: f32,
    /// 音乐音量
    pub music_volume: f32,
    /// 音效音量
    pub sfx_volume: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            difficulty: DifficultyLevel::Normal,
            master_volume: 0.8,
            music_volume: 0.5,
            sfx_volume: 0.7,
        }
    }
}

impl From<&SettingsData> for GameSettings {
    fn from(data: &SettingsData) -> Self {
        Self {
            difficulty: DifficultyLevel::from_str(&data.difficulty),
            master_volume: data.master_volume,
            music_volume: data.music_volume,
            sfx_volume: data.sfx_volume,
        }
    }
}

impl From<&GameSettings> for SettingsData {
    fn from(settings: &GameSettings) -> Self {
        Self {
            difficulty: settings.difficulty.to_string(),
            master_volume: settings.master_volume,
            music_volume: settings.music_volume,
            sfx_volume: settings.sfx_volume,
        }
    }
}

/// 设置UI标记
#[derive(Component)]
struct SettingsUI;

/// 设置选项类型
#[derive(Component)]
enum SettingsOption {
    Difficulty,
    MasterVolume,
    MusicVolume,
    SfxVolume,
}

/// 当前选中的设置选项
#[derive(Resource, Default)]
struct SelectedOption(usize);

/// 设置选项数量
const OPTION_COUNT: usize = 4;

/// 设置菜单
fn setup_settings(mut commands: Commands, settings: Res<GameSettings>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
            SettingsUI,
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new("SETTINGS"),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // 难度选项
            let diff_text = format!("Difficulty: {}", settings.difficulty.name());
            parent.spawn((
                Text::new(diff_text),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
                SettingsOption::Difficulty,
            ));

            // 主音量
            let master_text = format!("Master Volume: {:.0}%", settings.master_volume * 100.0);
            parent.spawn((
                Text::new(master_text),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
                SettingsOption::MasterVolume,
            ));

            // 音乐音量
            let music_text = format!("Music Volume: {:.0}%", settings.music_volume * 100.0);
            parent.spawn((
                Text::new(music_text),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
                SettingsOption::MusicVolume,
            ));

            // 音效音量
            let sfx_text = format!("SFX Volume: {:.0}%", settings.sfx_volume * 100.0);
            parent.spawn((
                Text::new(sfx_text),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
                SettingsOption::SfxVolume,
            ));

            // 操作提示
            parent.spawn((
                Text::new("Up/Down: Select  |  Left/Right: Adjust  |  ESC: Back"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node::default(),
            ));
        });

    commands.insert_resource(SelectedOption(0));
}

/// 清理设置菜单
fn cleanup_settings(mut commands: Commands, query: Query<Entity, With<SettingsUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<SelectedOption>();
}

/// 设置系统
fn settings_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<GameSettings>,
    mut save_data: ResMut<SaveData>,
    mut selected: ResMut<SelectedOption>,
    mut query: Query<(&mut Text, &SettingsOption, &mut TextColor)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // 上下选择
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        selected.0 = if selected.0 == 0 { OPTION_COUNT - 1 } else { selected.0 - 1 };
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        selected.0 = (selected.0 + 1) % OPTION_COUNT;
    }

    // 左右调整
    if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::ArrowRight) {
        let delta = if keyboard.just_pressed(KeyCode::ArrowRight) { 1 } else { -1 };

        match selected.0 {
            0 => {
                // 难度循环
                settings.difficulty = match settings.difficulty {
                    DifficultyLevel::Easy => if delta > 0 { DifficultyLevel::Normal } else { DifficultyLevel::Hard },
                    DifficultyLevel::Normal => if delta > 0 { DifficultyLevel::Hard } else { DifficultyLevel::Easy },
                    DifficultyLevel::Hard => if delta > 0 { DifficultyLevel::Easy } else { DifficultyLevel::Normal },
                };
            }
            1 => {
                // 主音量
                settings.master_volume = (settings.master_volume + delta as f32 * 0.1).clamp(0.0, 1.0);
            }
            2 => {
                // 音乐音量
                settings.music_volume = (settings.music_volume + delta as f32 * 0.1).clamp(0.0, 1.0);
            }
            3 => {
                // 音效音量
                settings.sfx_volume = (settings.sfx_volume + delta as f32 * 0.1).clamp(0.0, 1.0);
            }
            _ => {}
        }

        // 保存设置到存档
        save_data.settings = SettingsData::from(&*settings);
        if let Err(e) = crate::save::save_to_file(&save_data) {
            warn!("设置保存失败: {}", e);
        }
    }

    // ESC 返回
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Menu);
    }

    // 更新UI显示和高亮
    let mut idx = 0;
    for (mut text, option, mut color) in query.iter_mut() {
        // 更新文本
        match option {
            SettingsOption::Difficulty => {
                **text = format!("Difficulty: {}", settings.difficulty.name());
            }
            SettingsOption::MasterVolume => {
                **text = format!("Master Volume: {:.0}%", settings.master_volume * 100.0);
            }
            SettingsOption::MusicVolume => {
                **text = format!("Music Volume: {:.0}%", settings.music_volume * 100.0);
            }
            SettingsOption::SfxVolume => {
                **text = format!("SFX Volume: {:.0}%", settings.sfx_volume * 100.0);
            }
        }

        // 高亮选中项
        *color = if idx == selected.0 {
            TextColor(Color::srgb(1.0, 1.0, 1.0))
        } else {
            TextColor(Color::srgb(0.7, 0.7, 0.7))
        };

        idx += 1;
    }
}
