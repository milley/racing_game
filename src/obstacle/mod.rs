use bevy::prelude::*;
use rand::Rng;

use crate::{
    GameConfig,
    game::{GameEntity, GameState, Difficulty, Combo},
    player::{Player, PlayerConfig},
    life::{PlayerLife, LifeConfig, handle_collision},
    powerup::ActivePowerUps,
    particle::{spawn_explosion, ParticleConfig},
    graphics::CarType,
};

/// 障碍物实体标记（公开供其他模块使用）
#[derive(Component)]
pub struct Obstacle;

/// 已被闪避标记
#[derive(Component)]
struct Dodged;

/// 障碍物碰撞箱组件
#[derive(Component)]
pub struct ObstacleHitbox {
    pub half_size: Vec2,
}

/// AABB 碰撞检测
/// 检测两个矩形是否碰撞
pub fn check_aabb_collision(
    pos_a: Vec2,
    half_size_a: Vec2,
    pos_b: Vec2,
    half_size_b: Vec2,
) -> bool {
    (pos_a.x - pos_b.x).abs() < half_size_a.x + half_size_b.x
        && (pos_a.y - pos_b.y).abs() < half_size_a.y + half_size_b.y
}

/// 障碍物插件
pub struct ObstaclePlugin;

impl Plugin for ObstaclePlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册障碍物资源
            .init_resource::<ObstacleConfig>()
            // 添加系统
            .add_systems(OnEnter(GameState::Playing), setup_obstacle_timer)
            .add_systems(
                Update,
                (
                    spawn_obstacles,
                    move_obstacles,
                    check_collisions,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// 判断障碍物是否应该被移除
pub fn should_despawn_obstacle(y_position: f32, screen_bottom: f32) -> bool {
    y_position < screen_bottom
}

/// 记录闪避（增加连击）
pub fn record_dodge(combo: &mut Combo) {
    combo.count += 1;
    combo.combo_multiplier = 1.0 + (combo.count as f32 * 0.1).min(2.0);
    combo.timer = combo.max_timer;
}

/// 碰撞重置连击
pub fn reset_combo_on_collision(combo: &mut Combo) {
    combo.count = 0;
    combo.combo_multiplier = 1.0;
    combo.timer = 0.0;
}

/// 障碍物配置
#[derive(Resource)]
struct ObstacleConfig {
    /// 障碍物宽度
    width: f32,
    /// 障碍物高度
    height: f32,
    /// 下落速度
    speed: f32,
    /// 生成间隔（秒）
    spawn_interval: f32,
}

impl Default for ObstacleConfig {
    fn default() -> Self {
        Self {
            width: 35.0,
            height: 50.0,
            speed: 250.0,
            spawn_interval: 1.5,
        }
    }
}

/// 障碍物生成计时器
#[derive(Resource)]
struct ObstacleTimer(Timer);

/// 设置障碍物计时器
fn setup_obstacle_timer(mut commands: Commands, config: Res<ObstacleConfig>) {
    commands.insert_resource(ObstacleTimer(Timer::from_seconds(
        config.spawn_interval,
        TimerMode::Repeating,
    )));
}

/// 生成障碍物
fn spawn_obstacles(
    mut commands: Commands,
    mut timer: ResMut<ObstacleTimer>,
    config: Res<ObstacleConfig>,
    game_config: Res<GameConfig>,
    difficulty: Res<Difficulty>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    // 根据难度调整生成间隔
    let adjusted_interval = config.spawn_interval * difficulty.spawn_interval_multiplier;
    timer.0.set_duration(std::time::Duration::from_secs_f32(adjusted_interval));

    if timer.0.just_finished() {
        let mut rng = rand::thread_rng();

        // 随机车辆类型
        let car_types = [CarType::Sedan, CarType::Suv, CarType::Truck, CarType::Sports];
        let car_type = car_types[rng.gen_range(0..car_types.len())];

        // 根据车辆类型获取尺寸
        let hitbox_size = car_type.hitbox_size();
        let half_road = game_config.road_width / 2.0 - hitbox_size.x / 2.0;

        let x = (rng.gen::<f32>() - 0.5) * 2.0 * half_road;
        let y = 400.0;

        commands.spawn((
            Transform::from_xyz(x, y, 1.0),
            Obstacle,
            GameEntity,
            crate::graphics::CarTypeComponent(car_type),
            ObstacleHitbox {
                half_size: hitbox_size / 2.0,
            },
        ));
    }
}

/// 移动障碍物
fn move_obstacles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, Option<&Dodged>), (With<Obstacle>, Without<Player>)>,
    config: Res<ObstacleConfig>,
    difficulty: Res<Difficulty>,
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Obstacle>)>,
    mut combo: ResMut<Combo>,
    active_powerups: Res<ActivePowerUps>,
) {
    // 根据难度调整速度
    let mut adjusted_speed = config.speed * difficulty.speed_multiplier;

    // 应用减速效果
    if active_powerups.has_slowdown {
        adjusted_speed *= 0.5; // 减速50%
    }

    // 获取玩家位置
    let player_y = player_query.single().map(|t| t.translation.y).unwrap_or(-220.0);

    for (entity, mut transform, dodged) in query.iter_mut() {
        transform.translation.y -= adjusted_speed * time.delta_secs();

        // 检测闪避：障碍物通过玩家下方且未被标记
        if transform.translation.y < player_y - 30.0 && dodged.is_none() {
            // 标记为已闪避
            commands.entity(entity).insert(Dodged);

            // 增加连击
            record_dodge(&mut combo);
        }

        // 移出屏幕后删除
        if transform.translation.y < -400.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// 碰撞检测
fn check_collisions(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut player_life: ResMut<PlayerLife>,
    life_config: Res<LifeConfig>,
    active_powerups: Res<ActivePowerUps>,
    particle_config: Res<ParticleConfig>,
    player_query: Query<&Transform, With<Player>>,
    obstacle_query: Query<(Entity, &Transform, &ObstacleHitbox), With<Obstacle>>,
    player_config: Res<PlayerConfig>,
    mut achievement_tracker: ResMut<crate::achievement::AchievementTracker>,
    mut save_data: ResMut<crate::save::SaveData>,
    mut combo: ResMut<Combo>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let mut player_half = Vec2::new(player_config.width / 2.0, player_config.height / 2.0);

    // 应用缩小效果
    if active_powerups.has_shrink {
        player_half *= 0.5;
    }

    for (obstacle_entity, obstacle_transform, hitbox) in obstacle_query.iter() {
        let obstacle_pos = obstacle_transform.translation.truncate();

        // AABB 碰撞检测
        if check_aabb_collision(player_pos, player_half, obstacle_pos, hitbox.half_size) {
            // 检查护盾
            if active_powerups.has_shield {
                // 有护盾，销毁障碍物但不受伤
                spawn_explosion(&mut commands, obstacle_transform.translation, &particle_config);
                commands.entity(obstacle_entity).despawn();
                continue;
            }

            // 记录碰撞成就
            crate::achievement::record_collision(&mut achievement_tracker, &mut save_data);

            // 生成碰撞粒子效果
            spawn_explosion(&mut commands, player_transform.translation, &particle_config);

            // 销毁障碍物
            commands.entity(obstacle_entity).despawn();

            // 重置连击
            reset_combo_on_collision(&mut combo);

            // 处理生命
            if handle_collision(&mut player_life, &life_config) {
                // 游戏结束
                next_state.set(GameState::GameOver);
            }
        }
    }
}
