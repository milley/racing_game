//! 道具系统
//!
//! 包含：
//! - 护盾道具：免疫一次碰撞
//! - 清除道具：清除所有障碍物

use bevy::prelude::*;
use rand::Rng;

use crate::game::{GameState, GameEntity, Difficulty};
use crate::player::Player;

/// 道具插件
pub struct PowerUpPlugin;

impl Plugin for PowerUpPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PowerUpConfig>()
            .init_resource::<ActivePowerUps>()
            .add_systems(OnEnter(GameState::Playing), setup_powerup_timer)
            .add_systems(Update, (
                spawn_powerups,
                move_powerups,
                collect_powerups,
                update_active_powerups,
            ).run_if(in_state(GameState::Playing)));
    }
}

/// 道具配置
#[derive(Resource)]
struct PowerUpConfig {
    /// 道具大小
    size: f32,
    /// 下落速度
    speed: f32,
    /// 生成间隔
    spawn_interval: f32,
    /// 护盾持续时间
    shield_duration: f32,
}

impl Default for PowerUpConfig {
    fn default() -> Self {
        Self {
            size: 30.0,
            speed: 150.0,
            spawn_interval: 8.0,
            shield_duration: 5.0,
        }
    }
}

/// 道具生成计时器
#[derive(Resource)]
struct PowerUpTimer(Timer);

/// 当前激活的道具效果
#[derive(Resource, Default)]
pub struct ActivePowerUps {
    /// 护盾剩余时间
    pub shield_timer: f32,
    /// 是否有护盾
    pub has_shield: bool,
}

/// 道具类型
#[derive(Component, Clone, Copy)]
enum PowerUpType {
    /// 护盾
    Shield,
    /// 清除障碍物
    Clear,
}

/// 道具实体标记
#[derive(Component)]
struct PowerUp;

/// 设置道具计时器
fn setup_powerup_timer(mut commands: Commands, config: Res<PowerUpConfig>) {
    commands.insert_resource(PowerUpTimer(Timer::from_seconds(
        config.spawn_interval,
        TimerMode::Repeating,
    )));
}

/// 生成道具
fn spawn_powerups(
    mut commands: Commands,
    mut timer: ResMut<PowerUpTimer>,
    config: Res<PowerUpConfig>,
    difficulty: Res<Difficulty>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    // 根据难度调整生成间隔
    let adjusted_interval = config.spawn_interval * difficulty.spawn_interval_multiplier;
    timer.0.set_duration(std::time::Duration::from_secs_f32(adjusted_interval));

    if timer.0.just_finished() {
        let mut rng = rand::thread_rng();

        // 随机位置
        let x = (rng.gen::<f32>() - 0.5) * 200.0;
        let y = 400.0;

        // 随机类型
        let powerup_type = if rng.gen::<bool>() {
            PowerUpType::Shield
        } else {
            PowerUpType::Clear
        };

        // 根据类型设置颜色和图标
        let (color, icon) = match powerup_type {
            PowerUpType::Shield => (Color::srgb(0.0, 0.8, 1.0), "🛡"),
            PowerUpType::Clear => (Color::srgb(1.0, 0.5, 0.0), "💥"),
        };

        commands.spawn((
            Sprite::from_color(color, Vec2::splat(config.size)),
            Transform::from_xyz(x, y, 1.0),
            PowerUp,
            powerup_type,
            GameEntity,
        ));

        // 显示图标（可选）
        commands.spawn((
            Text::new(icon),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            PowerUpIcon,
        ));
    }
}

/// 移动道具
fn move_powerups(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform), With<PowerUp>>,
    config: Res<PowerUpConfig>,
    time: Res<Time>,
) {
    for (entity, mut transform) in query.iter_mut() {
        transform.translation.y -= config.speed * time.delta_secs();

        if transform.translation.y < -400.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// 收集道具
fn collect_powerups(
    mut commands: Commands,
    mut active_powerups: ResMut<ActivePowerUps>,
    player_query: Query<&Transform, With<Player>>,
    powerup_query: Query<(Entity, &Transform, &PowerUpType), With<PowerUp>>,
    config: Res<PowerUpConfig>,
    obstacle_query: Query<Entity, With<crate::obstacle::Obstacle>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let player_half = Vec2::new(20.0, 30.0); // 玩家半尺寸

    for (entity, transform, powerup_type) in powerup_query.iter() {
        let powerup_pos = transform.translation.truncate();
        let powerup_half = Vec2::splat(config.size / 2.0);

        // 碰撞检测
        if (player_pos.x - powerup_pos.x).abs() < player_half.x + powerup_half.x
            && (player_pos.y - powerup_pos.y).abs() < player_half.y + powerup_half.y
        {
            // 收集道具
            match powerup_type {
                PowerUpType::Shield => {
                    active_powerups.has_shield = true;
                    active_powerups.shield_timer = config.shield_duration;
                }
                PowerUpType::Clear => {
                    // 清除所有障碍物
                    for obstacle_entity in obstacle_query.iter() {
                        commands.entity(obstacle_entity).despawn();
                    }
                }
            }

            commands.entity(entity).despawn();
        }
    }
}

/// 更新激活的道具效果
fn update_active_powerups(
    mut commands: Commands,
    mut active_powerups: ResMut<ActivePowerUps>,
    player_query: Query<&Transform, (With<Player>, Without<ShieldVisual>)>,
    mut shield_query: Query<(Entity, &mut Transform, &mut Sprite), (With<ShieldVisual>, Without<Player>)>,
    time: Res<Time>,
) {
    if active_powerups.has_shield {
        active_powerups.shield_timer -= time.delta_secs();

        if active_powerups.shield_timer <= 0.0 {
            active_powerups.has_shield = false;
            // 删除护盾框
            for (entity, _, _) in shield_query.iter() {
                commands.entity(entity).despawn();
            }
        } else {
            // 护盾闪烁效果
            let pulse = (time.elapsed_secs() * 4.0).sin() * 0.3 + 0.7;

            // 如果护盾框不存在，创建它
            if shield_query.is_empty() {
                if let Ok(player_transform) = player_query.single() {
                    // 创建护盾框（四个边）
                    let shield_size = Vec2::new(50.0, 70.0);
                    let thickness = 3.0;
                    let px = player_transform.translation.x;
                    let py = player_transform.translation.y;

                    // 上边
                    commands.spawn((
                        Sprite::from_color(Color::srgba(0.0, 0.8, 1.0, 0.8), Vec2::new(shield_size.x, thickness)),
                        Transform::from_xyz(px, py + shield_size.y / 2.0, 1.5),
                        ShieldVisual,
                    ));

                    // 下边
                    commands.spawn((
                        Sprite::from_color(Color::srgba(0.0, 0.8, 1.0, 0.8), Vec2::new(shield_size.x, thickness)),
                        Transform::from_xyz(px, py - shield_size.y / 2.0, 1.5),
                        ShieldVisual,
                    ));

                    // 左边
                    commands.spawn((
                        Sprite::from_color(Color::srgba(0.0, 0.8, 1.0, 0.8), Vec2::new(thickness, shield_size.y)),
                        Transform::from_xyz(px - shield_size.x / 2.0, py, 1.5),
                        ShieldVisual,
                    ));

                    // 右边
                    commands.spawn((
                        Sprite::from_color(Color::srgba(0.0, 0.8, 1.0, 0.8), Vec2::new(thickness, shield_size.y)),
                        Transform::from_xyz(px + shield_size.x / 2.0, py, 1.5),
                        ShieldVisual,
                    ));
                }
            } else {
                // 更新护盾框位置和闪烁
                if let Ok(player_transform) = player_query.single() {
                    let px = player_transform.translation.x;
                    let py = player_transform.translation.y;
                    let shield_size = Vec2::new(50.0, 70.0);
                    let positions = [
                        (px, py + shield_size.y / 2.0),   // 上
                        (px, py - shield_size.y / 2.0),   // 下
                        (px - shield_size.x / 2.0, py),   // 左
                        (px + shield_size.x / 2.0, py),   // 右
                    ];

                    for (i, (_, mut transform, mut sprite)) in shield_query.iter_mut().enumerate() {
                        if i < positions.len() {
                            transform.translation.x = positions[i].0;
                            transform.translation.y = positions[i].1;
                            sprite.color = Color::srgba(0.0, 0.8, 1.0, 0.5 + pulse * 0.5);
                        }
                    }
                }
            }
        }
    }
}

/// 护盾视觉效果标记
#[derive(Component)]
struct ShieldVisual;

/// 道具图标标记
#[derive(Component)]
struct PowerUpIcon;
