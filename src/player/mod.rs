use bevy::prelude::*;

use crate::{GameConfig, game::{GameEntity, GameState}};

/// 玩家插件
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册玩家资源
            .init_resource::<PlayerConfig>()
            // 添加系统
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                player_movement.run_if(in_state(GameState::Playing)),
            );
    }
}

/// 玩家配置
#[derive(Resource)]
pub struct PlayerConfig {
    /// 移动速度
    pub speed: f32,
    /// 玩家宽度
    pub width: f32,
    /// 玩家高度
    pub height: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            speed: 300.0,
            width: 40.0,
            height: 60.0,
        }
    }
}

/// 玩家实体标记
#[derive(Component)]
pub struct Player;

/// 玩家位置（用于碰撞检测）
#[derive(Component)]
struct PlayerPosition(Vec2);

/// 生成玩家
fn spawn_player(
    mut commands: Commands,
    _config: Res<PlayerConfig>,
    _game_config: Res<GameConfig>,
) {
    let start_x = 0.0;
    let start_y = -220.0;

    // 玩家实体（由像素图形系统处理显示，不需要 Sprite）
    commands.spawn((
        Transform::from_xyz(start_x, start_y, 1.0),
        Visibility::default(),
        Player,
        PlayerPosition(Vec2::new(start_x, start_y)),
        GameEntity,
    ));
}

/// 玩家移动
fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    config: Res<PlayerConfig>,
    game_config: Res<GameConfig>,
    mut query: Query<(&mut Transform, &mut PlayerPosition), With<Player>>,
    time: Res<Time>,
) {
    let Ok((mut transform, mut position)) = query.single_mut() else {
        return;
    };

    let delta = config.speed * time.delta_secs();

    // 计算道路边界
    let half_road = game_config.road_width / 2.0 - config.width / 2.0;

    // 移动逻辑
    let mut direction = 0.0;
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        direction -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        direction += 1.0;
    }

    // 更新位置
    position.0.x += direction * delta;
    position.0.x = position.0.x.clamp(-half_road, half_road);

    transform.translation.x = position.0.x;
}
