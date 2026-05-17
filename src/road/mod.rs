use bevy::prelude::*;

use crate::{GameConfig, game::{GameEntity, GameState, Difficulty}};

/// 道路插件
pub struct RoadPlugin;

impl Plugin for RoadPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册道路资源
            .init_resource::<RoadConfig>()
            // 添加系统
            .add_systems(OnEnter(GameState::Playing), spawn_road)
            .add_systems(
                Update,
                (update_road_lines, update_road_edges)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// 道路配置
#[derive(Resource)]
struct RoadConfig {
    /// 道路标线宽度
    line_width: f32,
    /// 道路标线高度
    line_height: f32,
    /// 标线间隔
    line_gap: f32,
    /// 标线移动速度
    scroll_speed: f32,
}

impl Default for RoadConfig {
    fn default() -> Self {
        Self {
            line_width: 8.0,
            line_height: 40.0,
            line_gap: 60.0,
            scroll_speed: 200.0,
        }
    }
}

/// 道路标线标记
#[derive(Component)]
struct RoadLine {
    /// 当前Y位置偏移
    offset: f32,
}

/// 道路边线标记
#[derive(Component)]
struct RoadEdge {
    /// 左或右
    is_left: bool,
}

/// 生成道路
fn spawn_road(mut commands: Commands, game_config: Res<GameConfig>, road_config: Res<RoadConfig>) {
    // 道路背景
    commands.spawn((
        Sprite::from_color(Color::srgb(0.2, 0.2, 0.2), Vec2::new(game_config.road_width, 800.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GameEntity,
    ));

    // 道路边界线（左）
    let left_edge_x = -game_config.road_width / 2.0 - 5.0;
    commands.spawn((
        Sprite::from_color(Color::srgb(1.0, 1.0, 1.0), Vec2::new(10.0, 800.0)),
        Transform::from_xyz(left_edge_x, 0.0, 0.5),
        RoadEdge { is_left: true },
        GameEntity,
    ));

    // 道路边界线（右）
    let right_edge_x = game_config.road_width / 2.0 + 5.0;
    commands.spawn((
        Sprite::from_color(Color::srgb(1.0, 1.0, 1.0), Vec2::new(10.0, 800.0)),
        Transform::from_xyz(right_edge_x, 0.0, 0.5),
        RoadEdge { is_left: false },
        GameEntity,
    ));

    // 中央虚线
    let total_height = road_config.line_height + road_config.line_gap;
    let line_count = (800.0 / total_height).ceil() as i32 + 1;

    for i in 0..line_count {
        let y = (i as f32 - line_count as f32 / 2.0) * total_height;
        commands.spawn((
            Sprite::from_color(
                Color::srgb(1.0, 1.0, 1.0),
                Vec2::new(road_config.line_width, road_config.line_height),
            ),
            Transform::from_xyz(0.0, y, 0.5),
            RoadLine { offset: y },
            GameEntity,
        ));
    }
}

/// 更新道路标线（滚动效果）
fn update_road_lines(
    mut query: Query<(&mut Transform, &mut RoadLine)>,
    road_config: Res<RoadConfig>,
    difficulty: Res<Difficulty>,
    time: Res<Time>,
) {
    let total_height = road_config.line_height + road_config.line_gap;
    // 根据难度调整滚动速度
    let adjusted_speed = road_config.scroll_speed * difficulty.speed_multiplier;

    for (mut transform, mut road_line) in query.iter_mut() {
        // 向下移动
        road_line.offset -= adjusted_speed * time.delta_secs();

        // 循环滚动
        if road_line.offset < -400.0 - total_height {
            road_line.offset += total_height * 10.0;
        }

        transform.translation.y = road_line.offset;
    }
}

/// 更新道路边线（可选：闪烁效果）
fn update_road_edges(
    mut query: Query<(&mut Sprite, &RoadEdge)>,
    time: Res<Time>,
) {
    // 闪烁效果：每0.5秒切换
    let flash = (time.elapsed_secs() * 2.0).floor() as i32 % 2 == 0;

    for (mut sprite, _edge) in query.iter_mut() {
        if flash {
            sprite.color = Color::srgb(1.0, 0.0, 0.0);
        } else {
            sprite.color = Color::srgb(1.0, 1.0, 1.0);
        }
    }
}
