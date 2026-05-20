//! 游戏模式系统
//!
//! 包含：
//! - Classic: 经典模式（3条命）
//! - Endless: 无尽模式（一碰即死）
//! - TimeAttack: 限时模式（60秒限时）

use bevy::prelude::*;

use crate::game::{GameState, Score, TimerText};

/// 游戏模式插件
pub struct GameModePlugin;

impl Plugin for GameModePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CurrentGameMode>()
            .init_resource::<TimeAttackTimer>()
            .add_systems(OnEnter(GameState::Playing), setup_game_mode)
            .add_systems(OnExit(GameState::Playing), cleanup_game_mode)
            .add_systems(Update, (
                update_time_attack_timer.run_if(in_state(GameState::Playing)),
            ));
    }
}

/// 游戏模式
#[derive(Resource, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum GameMode {
    /// 经典模式：3条命，无时间限制
    #[default]
    Classic,
    /// 无尽模式：1条命，一碰即死
    Endless,
    /// 限时模式：60秒限时，追求最高分
    TimeAttack,
}

/// 当前游戏模式
#[derive(Resource, Default)]
pub struct CurrentGameMode {
    pub mode: GameMode,
}

/// 限时模式计时器
#[derive(Resource)]
pub struct TimeAttackTimer {
    /// 剩余时间（秒）
    pub remaining: f32,
    /// 总时间（秒）
    pub total: f32,
    /// 是否激活
    pub active: bool,
}

impl Default for TimeAttackTimer {
    fn default() -> Self {
        Self {
            remaining: 60.0,
            total: 60.0,
            active: false,
        }
    }
}

/// 设置游戏模式
fn setup_game_mode(
    mut time_attack_timer: ResMut<TimeAttackTimer>,
    current_mode: Res<CurrentGameMode>,
) {
    match current_mode.mode {
        GameMode::TimeAttack => {
            time_attack_timer.remaining = 60.0;
            time_attack_timer.active = true;
        }
        _ => {
            time_attack_timer.active = false;
        }
    }
}

/// 清理游戏模式
fn cleanup_game_mode(mut time_attack_timer: ResMut<TimeAttackTimer>) {
    time_attack_timer.active = false;
}

/// 更新限时模式计时器
fn update_time_attack_timer(
    mut time_attack_timer: ResMut<TimeAttackTimer>,
    mut next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
    mut score: ResMut<Score>,
    mut save_data: ResMut<crate::save::SaveData>,
    mut achievement_tracker: ResMut<crate::achievement::AchievementTracker>,
    mut query: Query<&mut Text, With<TimerText>>,
) {
    if !time_attack_timer.active {
        return;
    }

    time_attack_timer.remaining -= time.delta_secs();

    // 更新UI显示
    for mut text in query.iter_mut() {
        **text = format!("Time: {:.1}s", time_attack_timer.remaining.max(0.0));
    }

    if time_attack_timer.remaining <= 0.0 {
        time_attack_timer.remaining = 0.0;
        time_attack_timer.active = false;

        // 更新最高分
        score.high_score = save_data.high_score;
        if score.value > score.high_score {
            score.high_score = score.value;
        }

        // 记录游戏结束成就
        crate::achievement::record_game_over(&mut achievement_tracker, &mut save_data);

        // 游戏结束
        next_state.set(GameState::GameOver);
    }
}

/// 获取模式显示名称
pub fn get_mode_name(mode: GameMode) -> &'static str {
    match mode {
        GameMode::Classic => "Classic",
        GameMode::Endless => "Endless",
        GameMode::TimeAttack => "Time Attack",
    }
}

/// 获取模式描述
pub fn get_mode_description(mode: GameMode) -> &'static str {
    match mode {
        GameMode::Classic => "3 Lives, No Time Limit",
        GameMode::Endless => "1 Life, One Hit = Game Over",
        GameMode::TimeAttack => "60 Seconds, Max Score",
    }
}
