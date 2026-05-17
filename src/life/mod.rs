//! 生命系统
//!
//! 玩家有3条生命，碰撞后短暂无敌

use bevy::prelude::*;

use crate::game::GameState;
use crate::player::Player;

/// 生命插件
pub struct LifePlugin;

impl Plugin for LifePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<LifeConfig>()
            .init_resource::<PlayerLife>()
            .add_systems(OnEnter(GameState::Playing), (reset_life, spawn_life_ui))
            .add_systems(Update, (
                update_invincibility,
                update_life_display,
            ).run_if(in_state(GameState::Playing)));
    }
}

/// 生命配置
#[derive(Resource)]
pub struct LifeConfig {
    /// 最大生命值
    pub max_lives: u32,
    /// 无敌时间（秒）
    pub invincibility_duration: f32,
}

impl Default for LifeConfig {
    fn default() -> Self {
        Self {
            max_lives: 3,
            invincibility_duration: 2.0,
        }
    }
}

/// 玩家生命资源
#[derive(Resource)]
pub struct PlayerLife {
    /// 当前生命值
    pub lives: u32,
    /// 是否无敌
    pub is_invincible: bool,
    /// 无敌剩余时间
    pub invincibility_timer: f32,
}

impl Default for PlayerLife {
    fn default() -> Self {
        Self {
            lives: 3,
            is_invincible: false,
            invincibility_timer: 0.0,
        }
    }
}

/// 重置生命
fn reset_life(mut player_life: ResMut<PlayerLife>, config: Res<LifeConfig>) {
    player_life.lives = config.max_lives;
    player_life.is_invincible = false;
    player_life.invincibility_timer = 0.0;
}

/// 生成生命UI
fn spawn_life_ui(mut commands: Commands) {
    commands.spawn((
        Text::new("Lives: 3"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.3, 0.3)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(10.0),
            ..default()
        },
        LifeUI,
    ));
}

/// 更新无敌状态
fn update_invincibility(
    mut player_life: ResMut<PlayerLife>,
    mut player_query: Query<&mut Sprite, With<Player>>,
    time: Res<Time>,
) {
    if player_life.is_invincible {
        player_life.invincibility_timer -= time.delta_secs();

        if player_life.invincibility_timer <= 0.0 {
            player_life.is_invincible = false;
            // 恢复可见
            if let Ok(mut sprite) = player_query.single_mut() {
                sprite.color.set_alpha(1.0);
            }
        } else {
            // 闪烁效果
            if let Ok(mut sprite) = player_query.single_mut() {
                let blink = (time.elapsed_secs() * 8.0).sin() > 0.0;
                sprite.color.set_alpha(if blink { 0.3 } else { 1.0 });
            }
        }
    }
}

/// 更新生命显示
fn update_life_display(
    player_life: Res<PlayerLife>,
    mut query: Query<&mut Text, With<LifeUI>>,
) {
    if player_life.is_changed() {
        if let Ok(mut text) = query.single_mut() {
            // 使用数字显示生命值
            **text = format!("Lives: {}", player_life.lives);
        }
    }
}

/// 处理碰撞
/// 返回是否应该游戏结束
pub fn handle_collision(
    player_life: &mut PlayerLife,
    config: &LifeConfig,
) -> bool {
    if player_life.is_invincible {
        return false;
    }

    if player_life.lives > 0 {
        player_life.lives -= 1;
    }

    if player_life.lives == 0 {
        true
    } else {
        player_life.is_invincible = true;
        player_life.invincibility_timer = config.invincibility_duration;
        false
    }
}

/// 生命UI标记
#[derive(Component)]
pub struct LifeUI;
