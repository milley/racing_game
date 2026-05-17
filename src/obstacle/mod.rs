use bevy::prelude::*;
use rand::Rng;

use crate::{GameConfig, game::{GameEntity, GameState}, player::{Player, PlayerConfig}};

/// AABB 碰撞检测
/// 检测两个矩形是否碰撞
#[inline]
fn check_aabb_collision(
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

/// 障碍物实体标记
#[derive(Component)]
struct Obstacle;

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
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        // 随机位置
        let half_road = game_config.road_width / 2.0 - config.width / 2.0;
        let mut rng = rand::thread_rng();
        let x = (rng.gen::<f32>() - 0.5) * 2.0 * half_road;
        let y = 400.0;

        // 随机颜色（不同类型的车）
        let colors = [
            Color::srgb(1.0, 0.0, 0.0),   // 红色
            Color::srgb(0.0, 1.0, 0.0),   // 绿色
            Color::srgb(1.0, 0.5, 0.0),   // 橙色
            Color::srgb(0.5, 0.0, 0.5),   // 紫色
        ];
        let color = colors[rng.gen_range(0..colors.len())];

        commands.spawn((
            Sprite::from_color(color, Vec2::new(config.width, config.height)),
            Transform::from_xyz(x, y, 1.0),
            Obstacle,
            GameEntity,
        ));
    }
}

/// 移动障碍物
fn move_obstacles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform), With<Obstacle>>,
    config: Res<ObstacleConfig>,
    time: Res<Time>,
) {
    for (entity, mut transform) in query.iter_mut() {
        transform.translation.y -= config.speed * time.delta_secs();

        // 移出屏幕后删除
        if transform.translation.y < -400.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// 碰撞检测
fn check_collisions(
    mut next_state: ResMut<NextState<GameState>>,
    player_query: Query<&Transform, With<Player>>,
    obstacle_query: Query<&Transform, With<Obstacle>>,
    config: Res<ObstacleConfig>,
    player_config: Res<PlayerConfig>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let player_half = Vec2::new(player_config.width / 2.0, player_config.height / 2.0);

    for obstacle_transform in obstacle_query.iter() {
        let obstacle_pos = obstacle_transform.translation.truncate();
        let obstacle_half = Vec2::new(config.width / 2.0, config.height / 2.0);

        // AABB 碰撞检测
        if check_aabb_collision(player_pos, player_half, obstacle_pos, obstacle_half) {
            // 碰撞！游戏结束
            next_state.set(GameState::GameOver);
        }
    }
}
