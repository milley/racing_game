//! 粒子效果系统
//!
//! 包含：
//! - 碰撞爆炸粒子
//! - 速度线效果
//! - 氮气尾焰效果

use bevy::prelude::*;
use rand::Rng;

use crate::game::{GameState, GameEntity, Difficulty};
use crate::player::Player;
use crate::powerup::ActivePowerUps;

/// 粒子插件
pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ParticleConfig>()
            .add_systems(OnEnter(GameState::Playing), spawn_speed_lines)
            .add_systems(OnExit(GameState::Playing), cleanup_particles)
            .add_systems(Update, (
                update_particles,
                update_speed_lines,
                update_nitro_trail,
            ).run_if(in_state(GameState::Playing)));
    }
}

/// 粒子配置
#[derive(Resource)]
pub struct ParticleConfig {
    /// 爆炸粒子数量
    explosion_count: usize,
    /// 粒子生命周期
    particle_lifetime: f32,
    /// 粒子初始速度
    particle_speed: f32,
}

impl Default for ParticleConfig {
    fn default() -> Self {
        Self {
            explosion_count: 20,
            particle_lifetime: 0.8,
            particle_speed: 150.0,
        }
    }
}

/// 爆炸粒子标记
#[derive(Component)]
pub struct ExplosionParticle {
    /// 剩余生命时间
    lifetime: f32,
    /// 速度
    velocity: Vec2,
}

/// 速度线标记
#[derive(Component)]
struct SpeedLine;

/// 生成爆炸粒子
pub fn spawn_explosion(
    commands: &mut Commands,
    position: Vec3,
    config: &ParticleConfig,
) {
    let mut rng = rand::thread_rng();

    for _ in 0..config.explosion_count {
        // 随机方向
        let angle = rng.gen::<f32>() * std::f32::consts::TAU;
        let speed = config.particle_speed * (0.5 + rng.gen::<f32>() * 0.5);
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

        // 随机颜色（暖色调）
        let colors = [
            Color::srgb(1.0, 0.8, 0.0),   // 黄色
            Color::srgb(1.0, 0.5, 0.0),   // 橙色
            Color::srgb(1.0, 0.2, 0.0),   // 红橙色
            Color::srgb(1.0, 1.0, 0.5),   // 浅黄
        ];
        let color = colors[rng.gen_range(0..colors.len())];

        // 随机大小
        let size = 3.0 + rng.gen::<f32>() * 5.0;

        // 爆炸粒子不需要 GameEntity 标记，它们有自己的生命周期管理
        commands.spawn((
            Sprite::from_color(color, Vec2::new(size, size)),
            Transform::from_translation(position),
            ExplosionParticle {
                lifetime: config.particle_lifetime,
                velocity,
            },
        ));
    }
}

/// 更新爆炸粒子
fn update_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut ExplosionParticle, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut particle, mut sprite) in query.iter_mut() {
        particle.lifetime -= time.delta_secs();

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // 移动
        transform.translation.x += particle.velocity.x * time.delta_secs();
        transform.translation.y += particle.velocity.y * time.delta_secs();

        // 减速
        particle.velocity *= 0.98;

        // 淡出
        let alpha = (particle.lifetime / 0.8).min(1.0);
        sprite.color.set_alpha(alpha);
    }
}

/// 清理粒子
fn cleanup_particles(mut commands: Commands, query: Query<Entity, With<ExplosionParticle>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 生成速度线
fn spawn_speed_lines(mut commands: Commands) {
    // 左侧速度线
    for i in 0..8 {
        let y = (i as f32 - 4.0) * 80.0;
        commands.spawn((
            Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.3), Vec2::new(2.0, 40.0)),
            Transform::from_xyz(-180.0, y, 0.5),
            SpeedLine,
            GameEntity,
        ));
    }

    // 右侧速度线
    for i in 0..8 {
        let y = (i as f32 - 4.0) * 80.0;
        commands.spawn((
            Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.3), Vec2::new(2.0, 40.0)),
            Transform::from_xyz(180.0, y, 0.5),
            SpeedLine,
            GameEntity,
        ));
    }
}

/// 更新速度线
fn update_speed_lines(
    mut query: Query<(&mut Transform, &mut Sprite), With<SpeedLine>>,
    difficulty: Res<Difficulty>,
    time: Res<Time>,
) {
    let speed = 300.0 * difficulty.speed_multiplier;

    for (mut transform, mut sprite) in query.iter_mut() {
        // 向下移动
        transform.translation.y -= speed * time.delta_secs();

        // 循环
        if transform.translation.y < -350.0 {
            transform.translation.y += 640.0;
        }

        // 根据速度调整透明度
        let alpha = 0.2 + (difficulty.speed_multiplier - 1.0) * 0.3;
        sprite.color.set_alpha(alpha.min(0.6));
    }
}

/// 氮气尾焰粒子标记
#[derive(Component)]
struct NitroFlame {
    lifetime: f32,
}

/// 更新氮气尾焰效果
fn update_nitro_trail(
    mut commands: Commands,
    player_query: Query<&Transform, (With<Player>, Without<NitroFlame>)>,
    active_powerups: Res<ActivePowerUps>,
    mut flame_query: Query<(Entity, &mut Transform, &mut Sprite, &mut NitroFlame), (With<NitroFlame>, Without<Player>)>,
    time: Res<Time>,
) {
    // 更新现有的尾焰粒子
    for (entity, mut transform, mut sprite, mut flame) in flame_query.iter_mut() {
        flame.lifetime -= time.delta_secs();

        if flame.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // 向上移动并扩散
        transform.translation.y += 100.0 * time.delta_secs();
        let scale = 1.0 + (0.3 - flame.lifetime) * 2.0;
        transform.scale = Vec3::splat(scale.max(0.1));

        // 淡出
        let alpha = (flame.lifetime / 0.3).min(1.0);
        sprite.color.set_alpha(alpha);
    }

    // 如果氮气激活，生成新的尾焰粒子
    if active_powerups.has_nitro {
        if let Ok(player_transform) = player_query.single() {
            let mut rng = rand::thread_rng();

            // 在玩家后方生成火焰粒子
            for _ in 0..2 {
                let x = player_transform.translation.x + (rng.gen::<f32>() - 0.5) * 20.0;
                let y = player_transform.translation.y - 35.0;
                let size = 5.0 + rng.gen::<f32>() * 8.0;

                // 随机火焰颜色
                let colors = [
                    Color::srgb(1.0, 0.4, 0.0),   // 橙色
                    Color::srgb(1.0, 0.6, 0.0),   // 黄橙色
                    Color::srgb(1.0, 0.2, 0.0),   // 红橙色
                    Color::srgb(1.0, 0.8, 0.2),   // 黄色
                ];
                let color = colors[rng.gen_range(0..colors.len())];

                commands.spawn((
                    Sprite::from_color(color, Vec2::splat(size)),
                    Transform::from_xyz(x, y, 0.8),
                    NitroFlame { lifetime: 0.3 },
                ));
            }
        }
    }
}
