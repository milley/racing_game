use bevy::prelude::*;

use crate::{GameConfig, game::{GameEntity, GameState, Difficulty}};

/// 道路插件
pub struct RoadPlugin;

impl Plugin for RoadPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册道路资源
            .init_resource::<RoadConfig>()
            .init_resource::<Curvature>()
            // 添加系统
            .add_systems(OnEnter(GameState::Playing), spawn_road)
            .add_systems(
                Update,
                (
                    update_road_lines,
                    update_roadside_decorations,
                    update_curvature,
                ).run_if(in_state(GameState::Playing)),
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

/// 弯道曲率（模拟弯道效果）
#[derive(Resource)]
pub struct Curvature {
    /// 当前曲率 (-1 到 1)
    pub value: f32,
    /// 目标曲率
    pub target: f32,
    /// 曲率变化速度
    pub transition_speed: f32,
}

impl Default for Curvature {
    fn default() -> Self {
        Self {
            value: 0.0,
            target: 0.0,
            transition_speed: 0.5,
        }
    }
}

/// 道路标线标记
#[derive(Component)]
struct RoadLine {
    /// 当前Y位置偏移
    offset: f32,
    /// 原始X位置
    base_x: f32,
}

/// 路边装饰标记
#[derive(Component)]
struct RoadsideDecoration {
    /// 原始Y位置
    base_y: f32,
    /// 是否左侧
    is_left: bool,
}

/// 生成道路
fn spawn_road(mut commands: Commands, game_config: Res<GameConfig>, road_config: Res<RoadConfig>) {
    // 背景（草地）
    commands.spawn((
        Sprite::from_color(Color::srgb(0.15, 0.4, 0.15), Vec2::new(600.0, 800.0)),
        Transform::from_xyz(0.0, 0.0, -0.5),
        GameEntity,
    ));

    // 道路背景（深灰色沥青）
    commands.spawn((
        Sprite::from_color(Color::srgb(0.25, 0.25, 0.25), Vec2::new(game_config.road_width, 800.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GameEntity,
    ));

    // 道路边界装饰（红白条纹）
    let stripe_height = 40.0_f32;
    let stripe_count = (800.0_f32 / stripe_height).ceil() as i32 + 1;

    // 左边界
    for i in 0..stripe_count {
        let y = (i as f32 - stripe_count as f32 / 2.0) * stripe_height;
        let is_red = i % 2 == 0;
        let color = if is_red {
            Color::srgb(0.9, 0.2, 0.2)
        } else {
            Color::srgb(0.9, 0.9, 0.9)
        };

        commands.spawn((
            Sprite::from_color(color, Vec2::new(15.0, stripe_height)),
            Transform::from_xyz(-game_config.road_width / 2.0 - 10.0, y, 0.3),
            RoadsideDecoration { base_y: y, is_left: true },
            GameEntity,
        ));
    }

    // 右边界
    for i in 0..stripe_count {
        let y = (i as f32 - stripe_count as f32 / 2.0) * stripe_height;
        let is_red = i % 2 == 0;
        let color = if is_red {
            Color::srgb(0.9, 0.2, 0.2)
        } else {
            Color::srgb(0.9, 0.9, 0.9)
        };

        commands.spawn((
            Sprite::from_color(color, Vec2::new(15.0, stripe_height)),
            Transform::from_xyz(game_config.road_width / 2.0 + 10.0, y, 0.3),
            RoadsideDecoration { base_y: y, is_left: false },
            GameEntity,
        ));
    }

    // 道路边线（白色实线）
    let left_edge_x = -game_config.road_width / 2.0 + 5.0;
    commands.spawn((
        Sprite::from_color(Color::srgb(1.0, 1.0, 1.0), Vec2::new(4.0, 800.0)),
        Transform::from_xyz(left_edge_x, 0.0, 0.5),
        GameEntity,
    ));

    let right_edge_x = game_config.road_width / 2.0 - 5.0;
    commands.spawn((
        Sprite::from_color(Color::srgb(1.0, 1.0, 1.0), Vec2::new(4.0, 800.0)),
        Transform::from_xyz(right_edge_x, 0.0, 0.5),
        GameEntity,
    ));

    // 中央虚线（黄色）
    let total_height = road_config.line_height + road_config.line_gap;
    let line_count = (800.0 / total_height).ceil() as i32 + 1;

    for i in 0..line_count {
        let y = (i as f32 - line_count as f32 / 2.0) * total_height;
        commands.spawn((
            Sprite::from_color(
                Color::srgb(1.0, 0.9, 0.2),
                Vec2::new(road_config.line_width, road_config.line_height),
            ),
            Transform::from_xyz(0.0, y, 0.5),
            RoadLine { offset: y, base_x: 0.0 },
            GameEntity,
        ));
    }

    // 车道分隔线（左右）
    let lane_offset = game_config.road_width / 4.0;
    for lane_x in [-lane_offset, lane_offset] {
        for i in 0..line_count {
            let y = (i as f32 - line_count as f32 / 2.0) * total_height;
            commands.spawn((
                Sprite::from_color(
                    Color::srgb(1.0, 1.0, 1.0),
                    Vec2::new(4.0, road_config.line_height * 0.8),
                ),
                Transform::from_xyz(lane_x, y, 0.5),
                RoadLine { offset: y, base_x: lane_x },
                GameEntity,
            ));
        }
    }
}

/// 更新道路标线（只滚动，不左右移动）
fn update_road_lines(
    mut query: Query<(&mut Transform, &mut RoadLine)>,
    road_config: Res<RoadConfig>,
    difficulty: Res<Difficulty>,
    time: Res<Time>,
) {
    let total_height = road_config.line_height + road_config.line_gap;
    let adjusted_speed = road_config.scroll_speed * difficulty.speed_multiplier;

    for (mut transform, mut road_line) in query.iter_mut() {
        // 只进行垂直滚动
        road_line.offset -= adjusted_speed * time.delta_secs();

        if road_line.offset < -400.0 - total_height {
            road_line.offset += total_height * 10.0;
        }

        transform.translation.y = road_line.offset;
        // 保持原始X位置，不随弯道移动
        transform.translation.x = road_line.base_x;
    }
}

/// 更新路边装饰（滚动 + 轻微弯道效果）
fn update_roadside_decorations(
    mut query: Query<(&mut Transform, &RoadsideDecoration)>,
    difficulty: Res<Difficulty>,
    curvature: Res<Curvature>,
    time: Res<Time>,
) {
    let speed = 200.0 * difficulty.speed_multiplier;

    for (mut transform, decoration) in query.iter_mut() {
        // 垂直滚动
        transform.translation.y -= speed * time.delta_secs();

        // 循环滚动
        if transform.translation.y < -400.0 {
            transform.translation.y += 800.0;
        }

        // 弯道效果：路边装饰根据曲率和Y位置轻微偏移
        // 远处的偏移更大，近处偏移小（模拟透视）
        let y_normalized = (transform.translation.y / 400.0).clamp(-1.0, 1.0);
        let perspective_factor = (1.0 - y_normalized.abs()) * 0.5 + 0.5; // 远处大，近处小
        let curve_offset = curvature.value * perspective_factor * 20.0;

        // 左右两侧偏移方向相反
        let direction = if decoration.is_left { -1.0 } else { 1.0 };
        transform.translation.x = direction * (150.0 + 10.0) + curve_offset;
    }
}

/// 更新弯道曲率
fn update_curvature(
    mut curvature: ResMut<Curvature>,
    time: Res<Time>,
) {
    // 平滑过渡到目标曲率
    let diff = curvature.target - curvature.value;
    curvature.value += diff * curvature.transition_speed * time.delta_secs();

    // 随机改变目标曲率（模拟弯道）
    if rand::random::<f32>() < 0.003 {
        curvature.target = (rand::random::<f32>() - 0.5) * 0.8;
    }
}
