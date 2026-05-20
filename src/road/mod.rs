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
    /// 曲率持续时间计时器
    curve_timer: f32,
}

impl Default for Curvature {
    fn default() -> Self {
        Self {
            value: 0.0,
            target: 0.0,
            transition_speed: 2.0,
            curve_timer: 0.0,
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
    let stripe_count = (800.0_f32 / stripe_height).ceil() as i32 + 2;

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
            RoadsideDecoration { is_left: true },
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
            RoadsideDecoration { is_left: false },
            GameEntity,
        ));
    }

    // 中央虚线（黄色）
    let total_height = road_config.line_height + road_config.line_gap;
    let line_count = (800.0 / total_height).ceil() as i32 + 2;

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

/// 计算弯道偏移
/// y_pos: 当前Y位置
/// curvature: 曲率值 (-1 到 1)
/// 返回X偏移量
fn calculate_curve_offset(y_pos: f32, curvature: f32) -> f32 {
    // 透视因子：远处(y大)偏移大，近处(y小)偏移小
    // y范围: -400(近) 到 400(远)
    let y_normalized = (y_pos + 400.0) / 800.0; // 0 到 1
    let perspective = y_normalized * y_normalized; // 二次曲线，远处变化更剧烈

    // 弯道偏移：远处偏移大
    curvature * perspective * 100.0
}

/// 更新道路标线（滚动 + 弯道效果）
fn update_road_lines(
    mut query: Query<(&mut Transform, &mut RoadLine)>,
    road_config: Res<RoadConfig>,
    difficulty: Res<Difficulty>,
    curvature: Res<Curvature>,
    time: Res<Time>,
) {
    let total_height = road_config.line_height + road_config.line_gap;
    let adjusted_speed = road_config.scroll_speed * difficulty.speed_multiplier;

    for (mut transform, mut road_line) in query.iter_mut() {
        // 垂直滚动
        road_line.offset -= adjusted_speed * time.delta_secs();

        // 循环滚动
        if road_line.offset < -450.0 {
            road_line.offset += total_height * 12.0;
        }

        transform.translation.y = road_line.offset;

        // 弯道效果：根据Y位置和曲率计算X偏移
        let curve_offset = calculate_curve_offset(road_line.offset, curvature.value);
        transform.translation.x = road_line.base_x + curve_offset;
    }
}

/// 更新路边装饰（滚动 + 弯道效果）
fn update_roadside_decorations(
    mut query: Query<(&mut Transform, &mut Sprite, &RoadsideDecoration)>,
    difficulty: Res<Difficulty>,
    curvature: Res<Curvature>,
    game_config: Res<GameConfig>,
    time: Res<Time>,
) {
    let speed = 200.0 * difficulty.speed_multiplier;
    let stripe_height = 40.0;

    for (mut transform, _sprite, decoration) in query.iter_mut() {
        // 垂直滚动
        transform.translation.y -= speed * time.delta_secs();

        // 循环滚动
        if transform.translation.y < -420.0 {
            transform.translation.y += stripe_height * 22.0;
        }

        // 弯道效果
        let curve_offset = calculate_curve_offset(transform.translation.y, curvature.value);
        let base_x = if decoration.is_left {
            -game_config.road_width / 2.0 - 10.0
        } else {
            game_config.road_width / 2.0 + 10.0
        };
        transform.translation.x = base_x + curve_offset;
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

    // 曲率持续时间
    curvature.curve_timer -= time.delta_secs();

    // 当曲率稳定后，随机改变目标曲率
    if curvature.curve_timer <= 0.0 && diff.abs() < 0.1 {
        // 随机选择新的弯道方向
        // 曲率范围: -1.5 到 1.5 (更大的弯道)
        curvature.target = if rand::random::<f32>() < 0.5 {
            (rand::random::<f32>() - 0.5) * 3.0  // -1.5 到 1.5
        } else {
            0.0  // 直道
        };
        // 设置持续时间（2-5秒）
        curvature.curve_timer = 2.0 + rand::random::<f32>() * 3.0;
    }
}
