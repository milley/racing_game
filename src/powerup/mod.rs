//! 道具系统
//!
//! 包含：
//! - 护盾道具：免疫一次碰撞
//! - 清除道具：清除所有障碍物
//! - 磁铁道具：吸引附近道具
//! - 减速道具：降低障碍物速度
//! - 双倍分数：分数倍率翻倍
//! - 缩小道具：减小玩家碰撞箱
//! - 氮气加速：临时速度提升

use bevy::prelude::*;
use rand::Rng;

use crate::game::{GameState, GameEntity, Difficulty};
use crate::player::Player;
use crate::audio::{play_powerup_sound, play_shield_sound, AudioAssets};
use crate::settings::GameSettings;

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

/// 道具碰撞检测
pub fn check_powerup_collision(
    player_pos: Vec2,
    player_half: Vec2,
    powerup_pos: Vec2,
    powerup_half: Vec2,
) -> bool {
    (player_pos.x - powerup_pos.x).abs() < player_half.x + powerup_half.x
        && (player_pos.y - powerup_pos.y).abs() < player_half.y + powerup_half.y
}

/// 更新护盾计时器，返回护盾是否仍然有效
pub fn update_shield_timer(active_powerups: &mut ActivePowerUps, delta_secs: f32) -> bool {
    if active_powerups.has_shield {
        active_powerups.shield_timer -= delta_secs;
        if active_powerups.shield_timer <= 0.0 {
            active_powerups.has_shield = false;
            return false;
        }
        return true;
    }
    false
}

/// 激活护盾
pub fn activate_shield(active_powerups: &mut ActivePowerUps, duration: f32) {
    active_powerups.has_shield = true;
    active_powerups.shield_timer = duration;
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
    /// 磁铁持续时间
    magnet_duration: f32,
    /// 磁铁吸引范围
    magnet_range: f32,
    /// 减速持续时间
    slowdown_duration: f32,
    /// 减速因子
    slowdown_factor: f32,
    /// 双倍分数持续时间
    double_score_duration: f32,
    /// 缩小持续时间
    shrink_duration: f32,
    /// 缩小因子
    shrink_factor: f32,
    /// 氮气持续时间
    nitro_duration: f32,
    /// 氮气速度加成
    nitro_speed_boost: f32,
}

impl Default for PowerUpConfig {
    fn default() -> Self {
        Self {
            size: 30.0,
            speed: 150.0,
            spawn_interval: 8.0,
            shield_duration: 5.0,
            magnet_duration: 8.0,
            magnet_range: 150.0,
            slowdown_duration: 6.0,
            slowdown_factor: 0.5,
            double_score_duration: 10.0,
            shrink_duration: 5.0,
            shrink_factor: 0.5,
            nitro_duration: 3.0,
            nitro_speed_boost: 0.5,
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
    /// 磁铁剩余时间
    pub magnet_timer: f32,
    /// 是否有磁铁
    pub has_magnet: bool,
    /// 减速剩余时间
    pub slowdown_timer: f32,
    /// 是否有减速
    pub has_slowdown: bool,
    /// 双倍分数剩余时间
    pub double_score_timer: f32,
    /// 是否有双倍分数
    pub has_double_score: bool,
    /// 缩小剩余时间
    pub shrink_timer: f32,
    /// 是否有缩小
    pub has_shrink: bool,
    /// 氮气剩余时间
    pub nitro_timer: f32,
    /// 是否有氮气
    pub has_nitro: bool,
}

/// 道具类型
#[derive(Component, Clone, Copy)]
enum PowerUpType {
    /// 护盾
    Shield,
    /// 清除障碍物
    Clear,
    /// 磁铁
    Magnet,
    /// 减速
    Slowdown,
    /// 双倍分数
    DoubleScore,
    /// 缩小
    Shrink,
    /// 氮气加速
    NitroBoost,
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

        // 随机类型 (加权随机)
        let powerup_type = match rng.gen_range(0..100) {
            0..=20 => PowerUpType::Shield,      // 21%
            21..=35 => PowerUpType::Clear,       // 15%
            36..=50 => PowerUpType::Magnet,      // 15%
            51..=65 => PowerUpType::Slowdown,    // 15%
            66..=80 => PowerUpType::DoubleScore, // 15%
            81..=90 => PowerUpType::Shrink,      // 10%
            _ => PowerUpType::NitroBoost,        // 10%
        };

        // 根据类型设置颜色
        let color = match powerup_type {
            PowerUpType::Shield => Color::srgb(0.0, 0.8, 1.0),      // 青色
            PowerUpType::Clear => Color::srgb(1.0, 0.5, 0.0),       // 橙色
            PowerUpType::Magnet => Color::srgb(0.8, 0.2, 0.8),      // 紫色
            PowerUpType::Slowdown => Color::srgb(0.2, 0.5, 1.0),    // 蓝色
            PowerUpType::DoubleScore => Color::srgb(1.0, 0.85, 0.0), // 金色
            PowerUpType::Shrink => Color::srgb(0.2, 0.9, 0.3),      // 绿色
            PowerUpType::NitroBoost => Color::srgb(1.0, 0.4, 0.0),  // 橙红色
        };

        commands.spawn((
            Sprite::from_color(color, Vec2::splat(config.size)),
            Transform::from_xyz(x, y, 1.0),
            PowerUp,
            powerup_type,
            GameEntity,
        ));
    }
}

/// 移动道具
fn move_powerups(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform), (With<PowerUp>, Without<Player>)>,
    config: Res<PowerUpConfig>,
    time: Res<Time>,
    active_powerups: Res<ActivePowerUps>,
    player_query: Query<&Transform, (With<Player>, Without<PowerUp>)>,
) {
    let player_pos = player_query.single().map(|t| t.translation.truncate()).ok();

    for (entity, mut transform) in query.iter_mut() {
        // 磁铁效果：吸引附近道具
        if active_powerups.has_magnet {
            if let Some(pp) = player_pos {
                let powerup_pos = transform.translation.truncate();
                let dist = pp.distance(powerup_pos);
                if dist < config.magnet_range {
                    let direction = (pp - powerup_pos).normalize();
                    let attract_speed = 300.0 * (1.0 - dist / config.magnet_range);
                    transform.translation.x += direction.x * attract_speed * time.delta_secs();
                    transform.translation.y += direction.y * attract_speed * time.delta_secs();
                }
            }
        }

        // 正常下落
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
    mut achievement_tracker: ResMut<crate::achievement::AchievementTracker>,
    mut save_data: ResMut<crate::save::SaveData>,
    audio_assets: Res<AudioAssets>,
    settings: Res<GameSettings>,
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
            // 记录道具收集成就
            crate::achievement::record_powerup_collection(&mut achievement_tracker, &mut save_data);

            // 收集道具
            match powerup_type {
                PowerUpType::Shield => {
                    active_powerups.has_shield = true;
                    active_powerups.shield_timer = config.shield_duration;
                    play_shield_sound(&mut commands, &audio_assets, &settings);
                }
                PowerUpType::Clear => {
                    // 清除所有障碍物
                    for obstacle_entity in obstacle_query.iter() {
                        commands.entity(obstacle_entity).try_despawn();
                    }
                }
                PowerUpType::Magnet => {
                    active_powerups.has_magnet = true;
                    active_powerups.magnet_timer = config.magnet_duration;
                }
                PowerUpType::Slowdown => {
                    active_powerups.has_slowdown = true;
                    active_powerups.slowdown_timer = config.slowdown_duration;
                }
                PowerUpType::DoubleScore => {
                    active_powerups.has_double_score = true;
                    active_powerups.double_score_timer = config.double_score_duration;
                }
                PowerUpType::Shrink => {
                    active_powerups.has_shrink = true;
                    active_powerups.shrink_timer = config.shrink_duration;
                }
                PowerUpType::NitroBoost => {
                    active_powerups.has_nitro = true;
                    active_powerups.nitro_timer = config.nitro_duration;
                }
            }

            // 播放道具收集音效
            play_powerup_sound(&mut commands, &audio_assets, &settings);

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
    // 更新护盾
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

    // 更新磁铁
    if active_powerups.has_magnet {
        active_powerups.magnet_timer -= time.delta_secs();
        if active_powerups.magnet_timer <= 0.0 {
            active_powerups.has_magnet = false;
        }
    }

    // 更新减速
    if active_powerups.has_slowdown {
        active_powerups.slowdown_timer -= time.delta_secs();
        if active_powerups.slowdown_timer <= 0.0 {
            active_powerups.has_slowdown = false;
        }
    }

    // 更新双倍分数
    if active_powerups.has_double_score {
        active_powerups.double_score_timer -= time.delta_secs();
        if active_powerups.double_score_timer <= 0.0 {
            active_powerups.has_double_score = false;
        }
    }

    // 更新缩小
    if active_powerups.has_shrink {
        active_powerups.shrink_timer -= time.delta_secs();
        if active_powerups.shrink_timer <= 0.0 {
            active_powerups.has_shrink = false;
        }
    }

    // 更新氮气
    if active_powerups.has_nitro {
        active_powerups.nitro_timer -= time.delta_secs();
        if active_powerups.nitro_timer <= 0.0 {
            active_powerups.has_nitro = false;
        }
    }
}

/// 护盾视觉效果标记
#[derive(Component)]
struct ShieldVisual;
