//! 像素风格图形渲染
//!
//! 使用程序化生成的像素风格图形替代简单矩形

use bevy::prelude::*;

use crate::game::GameState;
use crate::player::Player;
use crate::obstacle::Obstacle;

/// 像素图形插件
pub struct PixelGraphicsPlugin;

impl Plugin for PixelGraphicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PixelConfig>()
            .add_systems(Update, (
                spawn_player_graphics,
                update_player_graphics,
                update_obstacle_graphics,
            ).run_if(in_state(GameState::Playing)));
    }
}

/// 像素配置
#[derive(Resource)]
pub struct PixelConfig {
    /// 像素大小
    pub pixel_size: f32,
}

impl Default for PixelConfig {
    fn default() -> Self {
        Self {
            pixel_size: 4.0,
        }
    }
}

/// 玩家车辆像素数据（赛车形状）
const PLAYER_CAR_PIXELS: &[(f32, f32, [f32; 4])] = &[
    // 车身主体（蓝色）
    // 底部
    (-3.0, 3.0, [0.0, 0.4, 0.8, 1.0]),
    (-2.0, 3.0, [0.0, 0.4, 0.8, 1.0]),
    (-1.0, 3.0, [0.0, 0.4, 0.8, 1.0]),
    (0.0, 3.0, [0.0, 0.4, 0.8, 1.0]),
    (1.0, 3.0, [0.0, 0.4, 0.8, 1.0]),
    (2.0, 3.0, [0.0, 0.4, 0.8, 1.0]),
    (3.0, 3.0, [0.0, 0.4, 0.8, 1.0]),

    (-4.0, 2.0, [0.0, 0.4, 0.8, 1.0]),
    (-3.0, 2.0, [0.0, 0.4, 0.8, 1.0]),
    (-2.0, 2.0, [0.0, 0.4, 0.8, 1.0]),
    (-1.0, 2.0, [0.0, 0.4, 0.8, 1.0]),
    (0.0, 2.0, [0.0, 0.4, 0.8, 1.0]),
    (1.0, 2.0, [0.0, 0.4, 0.8, 1.0]),
    (2.0, 2.0, [0.0, 0.4, 0.8, 1.0]),
    (3.0, 2.0, [0.0, 0.4, 0.8, 1.0]),
    (4.0, 2.0, [0.0, 0.4, 0.8, 1.0]),

    (-4.0, 1.0, [0.0, 0.4, 0.8, 1.0]),
    (-3.0, 1.0, [0.0, 0.4, 0.8, 1.0]),
    (3.0, 1.0, [0.0, 0.4, 0.8, 1.0]),
    (4.0, 1.0, [0.0, 0.4, 0.8, 1.0]),

    (-4.0, 0.0, [0.0, 0.4, 0.8, 1.0]),
    (-3.0, 0.0, [0.0, 0.4, 0.8, 1.0]),
    (-2.0, 0.0, [0.0, 0.4, 0.8, 1.0]),
    (-1.0, 0.0, [0.0, 0.4, 0.8, 1.0]),
    (0.0, 0.0, [0.0, 0.4, 0.8, 1.0]),
    (1.0, 0.0, [0.0, 0.4, 0.8, 1.0]),
    (2.0, 0.0, [0.0, 0.4, 0.8, 1.0]),
    (3.0, 0.0, [0.0, 0.4, 0.8, 1.0]),
    (4.0, 0.0, [0.0, 0.4, 0.8, 1.0]),

    (-4.0, -1.0, [0.0, 0.4, 0.8, 1.0]),
    (-3.0, -1.0, [0.0, 0.4, 0.8, 1.0]),
    (3.0, -1.0, [0.0, 0.4, 0.8, 1.0]),
    (4.0, -1.0, [0.0, 0.4, 0.8, 1.0]),

    (-4.0, -2.0, [0.0, 0.4, 0.8, 1.0]),
    (-3.0, -2.0, [0.0, 0.4, 0.8, 1.0]),
    (-2.0, -2.0, [0.0, 0.4, 0.8, 1.0]),
    (-1.0, -2.0, [0.0, 0.4, 0.8, 1.0]),
    (0.0, -2.0, [0.0, 0.4, 0.8, 1.0]),
    (1.0, -2.0, [0.0, 0.4, 0.8, 1.0]),
    (2.0, -2.0, [0.0, 0.4, 0.8, 1.0]),
    (3.0, -2.0, [0.0, 0.4, 0.8, 1.0]),
    (4.0, -2.0, [0.0, 0.4, 0.8, 1.0]),

    (-3.0, -3.0, [0.0, 0.4, 0.8, 1.0]),
    (-2.0, -3.0, [0.0, 0.4, 0.8, 1.0]),
    (-1.0, -3.0, [0.0, 0.4, 0.8, 1.0]),
    (0.0, -3.0, [0.0, 0.4, 0.8, 1.0]),
    (1.0, -3.0, [0.0, 0.4, 0.8, 1.0]),
    (2.0, -3.0, [0.0, 0.4, 0.8, 1.0]),
    (3.0, -3.0, [0.0, 0.4, 0.8, 1.0]),

    // 车窗（浅蓝色）
    (-2.0, 2.0, [0.4, 0.7, 1.0, 1.0]),
    (-1.0, 2.0, [0.4, 0.7, 1.0, 1.0]),
    (0.0, 2.0, [0.4, 0.7, 1.0, 1.0]),
    (1.0, 2.0, [0.4, 0.7, 1.0, 1.0]),
    (2.0, 2.0, [0.4, 0.7, 1.0, 1.0]),

    // 轮子（黑色）
    (-4.0, 1.0, [0.2, 0.2, 0.2, 1.0]),
    (4.0, 1.0, [0.2, 0.2, 0.2, 1.0]),
    (-4.0, -2.0, [0.2, 0.2, 0.2, 1.0]),
    (4.0, -2.0, [0.2, 0.2, 0.2, 1.0]),

    // 车头灯（黄色）
    (-2.0, 3.0, [1.0, 1.0, 0.5, 1.0]),
    (2.0, 3.0, [1.0, 1.0, 0.5, 1.0]),
];

/// 障碍物车辆类型
#[derive(Clone, Copy)]
pub enum CarType {
    /// 普通车（红色）
    Sedan,
    /// SUV（绿色）
    Suv,
    /// 卡车（橙色）
    Truck,
    /// 跑车（紫色）
    Sports,
}

impl CarType {
    /// 获取车辆颜色
    pub fn base_color(&self) -> [f32; 4] {
        match self {
            CarType::Sedan => [0.8, 0.2, 0.2, 1.0],
            CarType::Suv => [0.2, 0.6, 0.2, 1.0],
            CarType::Truck => [0.9, 0.5, 0.1, 1.0],
            CarType::Sports => [0.6, 0.2, 0.7, 1.0],
        }
    }

    /// 获取车辆尺寸倍率
    pub fn size_multiplier(&self) -> Vec2 {
        match self {
            CarType::Sedan => Vec2::new(1.0, 1.0),
            CarType::Suv => Vec2::new(1.1, 1.2),
            CarType::Truck => Vec2::new(1.2, 1.4),
            CarType::Sports => Vec2::new(0.9, 1.1),
        }
    }

    /// 获取碰撞箱尺寸
    pub fn hitbox_size(&self) -> Vec2 {
        let mult = self.size_multiplier();
        Vec2::new(35.0 * mult.x, 50.0 * mult.y)
    }
}

/// 获取障碍物车辆像素数据
fn get_obstacle_car_pixels(car_type: CarType) -> Vec<(f32, f32, [f32; 4])> {
    let base = car_type.base_color();
    let darker = [base[0] * 0.7, base[1] * 0.7, base[2] * 0.7, 1.0];

    vec![
        // 车身
        (-2.0, 2.5, darker),
        (-1.0, 2.5, base),
        (0.0, 2.5, base),
        (1.0, 2.5, base),
        (2.0, 2.5, darker),

        (-3.0, 1.5, base),
        (-2.0, 1.5, base),
        (-1.0, 1.5, base),
        (0.0, 1.5, base),
        (1.0, 1.5, base),
        (2.0, 1.5, base),
        (3.0, 1.5, base),

        (-3.0, 0.5, base),
        (3.0, 0.5, base),

        (-3.0, -0.5, base),
        (-2.0, -0.5, base),
        (-1.0, -0.5, base),
        (0.0, -0.5, base),
        (1.0, -0.5, base),
        (2.0, -0.5, base),
        (3.0, -0.5, base),

        (-3.0, -1.5, base),
        (3.0, -1.5, base),

        (-2.0, -2.5, darker),
        (-1.0, -2.5, base),
        (0.0, -2.5, base),
        (1.0, -2.5, base),
        (2.0, -2.5, darker),

        // 车窗
        (-1.0, 1.5, [0.3, 0.5, 0.7, 1.0]),
        (0.0, 1.5, [0.3, 0.5, 0.7, 1.0]),
        (1.0, 1.5, [0.3, 0.5, 0.7, 1.0]),

        // 轮子
        (-3.0, 0.5, [0.15, 0.15, 0.15, 1.0]),
        (3.0, 0.5, [0.15, 0.15, 0.15, 1.0]),
        (-3.0, -1.5, [0.15, 0.15, 0.15, 1.0]),
        (3.0, -1.5, [0.15, 0.15, 0.15, 1.0]),

        // 尾灯
        (-2.0, -2.5, [1.0, 0.3, 0.3, 1.0]),
        (2.0, -2.5, [1.0, 0.3, 0.3, 1.0]),
    ]
}

/// 像素图形组件
#[derive(Component)]
pub struct PixelGraphics {
    /// 子实体列表
    pub pixels: Vec<Entity>,
}

/// 车辆类型组件
#[derive(Component)]
pub struct CarTypeComponent(pub CarType);

/// 生成玩家车辆像素图形
fn spawn_player_graphics(
    mut commands: Commands,
    config: Res<PixelConfig>,
    player_query: Query<(Entity, Option<&PixelGraphics>), With<Player>>,
) {
    for (player_entity, graphics_opt) in player_query.iter() {
        // 如果已经有图形，跳过
        if graphics_opt.is_some() {
            continue;
        }

        let pixel_size = config.pixel_size;
        let mut pixel_entities = Vec::new();

        // 为每个像素生成一个精灵（不添加 GameEntity，随父实体一起删除）
        for (px, py, color) in PLAYER_CAR_PIXELS {
            let entity = commands.spawn((
                Sprite::from_color(
                    Color::srgba(color[0], color[1], color[2], color[3]),
                    Vec2::splat(pixel_size),
                ),
                Transform::from_xyz(px * pixel_size, py * pixel_size, 0.1),
                PixelPart,
            )).id();
            pixel_entities.push(entity);
        }

        // 将像素作为玩家的子实体
        for pixel_entity in &pixel_entities {
            commands.entity(player_entity).add_child(*pixel_entity);
        }

        commands.entity(player_entity).insert(PixelGraphics {
            pixels: pixel_entities,
        });
    }
}

/// 像素部件标记
#[derive(Component)]
pub struct PixelPart;

/// 更新玩家图形（无敌闪烁）
fn update_player_graphics(
    player_life: Res<crate::life::PlayerLife>,
    player_query: Query<&PixelGraphics, With<Player>>,
    mut pixel_query: Query<&mut Sprite, With<PixelPart>>,
    time: Res<Time>,
) {
    if let Ok(graphics) = player_query.single() {
        if player_life.is_invincible {
            let blink = (time.elapsed_secs() * 8.0).sin() > 0.0;
            let alpha = if blink { 0.3 } else { 1.0 };

            for &pixel_entity in &graphics.pixels {
                if let Ok(mut sprite) = pixel_query.get_mut(pixel_entity) {
                    sprite.color.set_alpha(alpha);
                }
            }
        } else {
            for &pixel_entity in &graphics.pixels {
                if let Ok(mut sprite) = pixel_query.get_mut(pixel_entity) {
                    sprite.color.set_alpha(1.0);
                }
            }
        }
    }
}

/// 更新障碍物图形
fn update_obstacle_graphics(
    mut commands: Commands,
    config: Res<PixelConfig>,
    obstacle_query: Query<(Entity, &Transform, &CarTypeComponent), (With<Obstacle>, Without<PixelGraphics>)>,
    existing_graphics_query: Query<&PixelGraphics, With<Obstacle>>,
) {
    // 如果已经有图形的障碍物，跳过
    if !existing_graphics_query.is_empty() {
        return;
    }

    let pixel_size = config.pixel_size;

    for (entity, _transform, car_type) in obstacle_query.iter() {
        let pixels = get_obstacle_car_pixels(car_type.0);
        let mut pixel_entities = Vec::new();
        let size_mult = car_type.0.size_multiplier();

        // 像素实体不添加 GameEntity，随父实体一起删除
        for (px, py, color) in &pixels {
            let pixel_entity = commands.spawn((
                Sprite::from_color(
                    Color::srgba(color[0], color[1], color[2], color[3]),
                    Vec2::splat(pixel_size),
                ),
                Transform::from_xyz(
                    px * pixel_size * size_mult.x,
                    py * pixel_size * size_mult.y,
                    0.1,
                ),
                PixelPart,
            )).id();
            pixel_entities.push(pixel_entity);
        }

        for pixel_entity in &pixel_entities {
            commands.entity(entity).add_child(*pixel_entity);
        }

        commands.entity(entity).insert(PixelGraphics {
            pixels: pixel_entities,
        });
    }
}
