//! 成就系统
//!
//! 追踪玩家成就并显示

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::game::GameState;
use crate::save::SaveData;

/// 成就插件
pub struct AchievementPlugin;

impl Plugin for AchievementPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AchievementTracker>()
            .add_systems(OnEnter(GameState::Achievements), setup_achievements_ui)
            .add_systems(OnExit(GameState::Achievements), cleanup_achievements_ui)
            .add_systems(Update, (
                achievements_system.run_if(in_state(GameState::Achievements)),
                check_achievements.run_if(in_state(GameState::Playing)),
            ));
    }
}

/// 成就ID
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum AchievementId {
    /// 首次碰撞
    FirstBlood,
    /// 存活1分钟
    Survivor,
    /// 达到难度等级5
    SpeedDemon,
    /// 收集10个道具
    Collector,
    /// 获得1000分
    HighScorer,
    /// 获得5000分
    Master,
    /// 获得10000分
    Legend,
    /// 游戏结束
    GameOver,
}

impl AchievementId {
    /// 获取成就名称
    pub fn name(&self) -> &'static str {
        match self {
            AchievementId::FirstBlood => "First Blood",
            AchievementId::Survivor => "Survivor",
            AchievementId::SpeedDemon => "Speed Demon",
            AchievementId::Collector => "Collector",
            AchievementId::HighScorer => "High Scorer",
            AchievementId::Master => "Master",
            AchievementId::Legend => "Legend",
            AchievementId::GameOver => "Game Over",
        }
    }

    /// 获取成就描述
    pub fn description(&self) -> &'static str {
        match self {
            AchievementId::FirstBlood => "First collision with an obstacle",
            AchievementId::Survivor => "Survive for 1 minute",
            AchievementId::SpeedDemon => "Reach difficulty level 5",
            AchievementId::Collector => "Collect 10 power-ups",
            AchievementId::HighScorer => "Score 1000 points",
            AchievementId::Master => "Score 5000 points",
            AchievementId::Legend => "Score 10000 points",
            AchievementId::GameOver => "Complete a game",
        }
    }

    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "FirstBlood" => Some(AchievementId::FirstBlood),
            "Survivor" => Some(AchievementId::Survivor),
            "SpeedDemon" => Some(AchievementId::SpeedDemon),
            "Collector" => Some(AchievementId::Collector),
            "HighScorer" => Some(AchievementId::HighScorer),
            "Master" => Some(AchievementId::Master),
            "Legend" => Some(AchievementId::Legend),
            "GameOver" => Some(AchievementId::GameOver),
            _ => None,
        }
    }

    /// 转换为字符串
    pub fn to_string(&self) -> String {
        match self {
            AchievementId::FirstBlood => "FirstBlood".to_string(),
            AchievementId::Survivor => "Survivor".to_string(),
            AchievementId::SpeedDemon => "SpeedDemon".to_string(),
            AchievementId::Collector => "Collector".to_string(),
            AchievementId::HighScorer => "HighScorer".to_string(),
            AchievementId::Master => "Master".to_string(),
            AchievementId::Legend => "Legend".to_string(),
            AchievementId::GameOver => "GameOver".to_string(),
        }
    }
}

/// 所有成就列表
pub const ALL_ACHIEVEMENTS: &[AchievementId] = &[
    AchievementId::FirstBlood,
    AchievementId::Survivor,
    AchievementId::SpeedDemon,
    AchievementId::Collector,
    AchievementId::HighScorer,
    AchievementId::Master,
    AchievementId::Legend,
    AchievementId::GameOver,
];

/// 成就追踪器
#[derive(Resource, Default)]
pub struct AchievementTracker {
    /// 已解锁的成就
    pub unlocked: HashSet<AchievementId>,
    /// 收集的道具数量（用于追踪）
    pub powerups_collected: u32,
    /// 是否有新解锁的成就（用于显示通知）
    pub new_unlock: Option<AchievementId>,
}

/// 成就UI标记
#[derive(Component)]
struct AchievementsUI;

/// 设置成就UI
fn setup_achievements_ui(mut commands: Commands, save_data: Res<SaveData>) {
    // 从存档加载已解锁的成就
    let unlocked: HashSet<AchievementId> = save_data.achievements
        .iter()
        .filter_map(|s| AchievementId::from_str(s))
        .collect();

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
            AchievementsUI,
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new("ACHIEVEMENTS"),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // 成就列表
            for achievement in ALL_ACHIEVEMENTS {
                let is_unlocked = unlocked.contains(achievement);
                let status = if is_unlocked { "[X]" } else { "[ ]" };
                let color = if is_unlocked {
                    Color::srgb(0.0, 1.0, 0.5)
                } else {
                    Color::srgb(0.5, 0.5, 0.5)
                };

                let text = format!("{} {} - {}", status, achievement.name(), achievement.description());
                parent.spawn((
                    Text::new(text),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(color),
                    Node {
                        margin: UiRect::bottom(Val::Px(5.0)),
                        ..default()
                    },
                ));
            }

            // 返回提示
            parent.spawn((
                Text::new("Press ESC to return"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            ));
        });
}

/// 清理成就UI
fn cleanup_achievements_ui(mut commands: Commands, query: Query<Entity, With<AchievementsUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 成就界面系统
fn achievements_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Menu);
    }
}

/// 检查成就条件
fn check_achievements(
    mut tracker: ResMut<AchievementTracker>,
    mut save_data: ResMut<SaveData>,
    score: Res<crate::game::Score>,
    game_timer: Res<crate::game::GameTimer>,
) {
    // 检查分数成就
    check_and_unlock(AchievementId::HighScorer, score.value >= 1000, &mut tracker, &mut save_data);
    check_and_unlock(AchievementId::Master, score.value >= 5000, &mut tracker, &mut save_data);
    check_and_unlock(AchievementId::Legend, score.value >= 10000, &mut tracker, &mut save_data);

    // 检查存活时间
    check_and_unlock(AchievementId::Survivor, game_timer.elapsed >= 60.0, &mut tracker, &mut save_data);

    // 检查难度等级
    let difficulty_level = (game_timer.elapsed / 10.0) as u32 + 1;
    check_and_unlock(AchievementId::SpeedDemon, difficulty_level >= 5, &mut tracker, &mut save_data);

    // 检查道具收集
    check_and_unlock(AchievementId::Collector, tracker.powerups_collected >= 10, &mut tracker, &mut save_data);
}

/// 检查并解锁成就
fn check_and_unlock(
    achievement: AchievementId,
    condition: bool,
    tracker: &mut AchievementTracker,
    save_data: &mut SaveData,
) {
    if condition && !tracker.unlocked.contains(&achievement) {
        tracker.unlocked.insert(achievement);
        tracker.new_unlock = Some(achievement);
        save_data.achievements.push(achievement.to_string());
        info!("成就解锁: {}", achievement.name());

        // 保存到文件
        if let Err(e) = crate::save::save_to_file(save_data) {
            warn!("成就保存失败: {}", e);
        }
    }
}

/// 记录道具收集
pub fn record_powerup_collection(
    tracker: &mut AchievementTracker,
    save_data: &mut SaveData,
) {
    tracker.powerups_collected += 1;
    check_and_unlock(AchievementId::Collector, tracker.powerups_collected >= 10, tracker, save_data);
}

/// 记录碰撞（首次碰撞成就）
pub fn record_collision(
    tracker: &mut AchievementTracker,
    save_data: &mut SaveData,
) {
    check_and_unlock(AchievementId::FirstBlood, true, tracker, save_data);
}

/// 记录游戏结束
pub fn record_game_over(
    tracker: &mut AchievementTracker,
    save_data: &mut SaveData,
) {
    check_and_unlock(AchievementId::GameOver, true, tracker, save_data);
}
