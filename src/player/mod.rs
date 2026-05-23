use bevy::prelude::*;
use crate::game::{GameState, GameEntity, Difficulty};
use crate::GameConfig;

/// 玩家实体标记
#[derive(Component)]
pub struct Player;

/// 玩家配置
#[derive(Resource)]
pub struct PlayerConfig {
    /// 玩家宽度
    pub width: f32,
    /// 玩家高度
    pub height: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            width: 40.0,
            height: 60.0,
        }
    }
}

/// 限制玩家位置在道路范围内
pub fn clamp_player_position(x: f32, road_width: f32, player_width: f32) -> f32 {
    let half_road = road_width / 2.0;
    let half_player = player_width / 2.0;
    x.clamp(-half_road + half_player, half_road - half_player)
}

/// 玩家移动
pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
    difficulty: Res<Difficulty>,
    game_config: Res<GameConfig>,
    player_config: Res<PlayerConfig>,
) {
    for mut transform in &mut query {
        let direction = if keyboard.pressed(KeyCode::ArrowLeft) {
            -1.0
        } else if keyboard.pressed(KeyCode::ArrowRight) {
            1.0
        } else {
            continue;
        };

        let speed = 300.0 * difficulty.speed_multiplier;
        let new_x = transform.translation.x + direction * speed * time.delta_secs();
        transform.translation.x = clamp_player_position(new_x, game_config.road_width, player_config.width);
    }
}

/// 创建玩家
pub fn spawn_player(
    mut commands: Commands,
) {
    commands.spawn((
        Transform::from_xyz(0.0, -200.0, 1.0),
        Player,
        GameEntity,
    ));
}

/// 玩家插件
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PlayerConfig>()
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(Update, player_movement.run_if(in_state(GameState::Playing)));
    }
}
