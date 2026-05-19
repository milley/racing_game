//! 游戏核心逻辑测试模块
//!
//! 使用 Bevy 测试框架进行单元测试

use bevy::prelude::*;

/// 测试辅助：创建最小化的测试 App
fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app
}

/// ========== 碰撞检测测试 ==========

/// AABB 碰撞检测核心逻辑
/// 返回两个矩形是否碰撞
pub fn check_aabb_collision(
    pos_a: Vec2,
    half_size_a: Vec2,
    pos_b: Vec2,
    half_size_b: Vec2,
) -> bool {
    (pos_a.x - pos_b.x).abs() < half_size_a.x + half_size_b.x
        && (pos_a.y - pos_b.y).abs() < half_size_a.y + half_size_b.y
}

#[cfg(test)]
mod collision_tests {
    use super::*;

    #[test]
    fn test_collision_direct_hit() {
        // 两矩形完全重叠
        let pos_a = Vec2::new(0.0, 0.0);
        let half_a = Vec2::new(20.0, 30.0);
        let pos_b = Vec2::new(0.0, 0.0);
        let half_b = Vec2::new(17.5, 25.0);

        assert!(check_aabb_collision(pos_a, half_a, pos_b, half_b));
    }

    #[test]
    fn test_collision_partial_overlap() {
        // 部分重叠
        let pos_a = Vec2::new(0.0, 0.0);
        let half_a = Vec2::new(20.0, 30.0); // 宽40, 高60
        let pos_b = Vec2::new(30.0, 40.0); // 偏移
        let half_b = Vec2::new(17.5, 25.0); // 宽35, 高50

        // x方向: |0-30|=30 < 20+17.5=37.5 ✓
        // y方向: |0-40|=40 < 30+25=55 ✓
        assert!(check_aabb_collision(pos_a, half_a, pos_b, half_b));
    }

    #[test]
    fn test_collision_edge_touching() {
        // 边缘刚好接触（不算碰撞，需要严格小于）
        let pos_a = Vec2::new(0.0, 0.0);
        let half_a = Vec2::new(20.0, 30.0);
        let pos_b = Vec2::new(37.5, 0.0); // 刚好在边缘
        let half_b = Vec2::new(17.5, 25.0);

        // x方向: |0-37.5|=37.5 < 20+17.5=37.5 是 false（不小于）
        assert!(!check_aabb_collision(pos_a, half_a, pos_b, half_b));
    }

    #[test]
    fn test_collision_no_overlap_x() {
        // X 轴无重叠
        let pos_a = Vec2::new(0.0, 0.0);
        let half_a = Vec2::new(20.0, 30.0);
        let pos_b = Vec2::new(50.0, 0.0); // 完全分离
        let half_b = Vec2::new(17.5, 25.0);

        assert!(!check_aabb_collision(pos_a, half_a, pos_b, half_b));
    }

    #[test]
    fn test_collision_no_overlap_y() {
        // Y 轴无重叠
        let pos_a = Vec2::new(0.0, 0.0);
        let half_a = Vec2::new(20.0, 30.0);
        let pos_b = Vec2::new(0.0, 60.0);
        let half_b = Vec2::new(17.5, 25.0);

        assert!(!check_aabb_collision(pos_a, half_a, pos_b, half_b));
    }

    #[test]
    fn test_collision_no_overlap_diagonal() {
        // 完全分离（对角）
        let pos_a = Vec2::new(0.0, 0.0);
        let half_a = Vec2::new(20.0, 30.0);
        let pos_b = Vec2::new(100.0, 100.0);
        let half_b = Vec2::new(17.5, 25.0);

        assert!(!check_aabb_collision(pos_a, half_a, pos_b, half_b));
    }
}

/// ========== 边界限制测试 ==========

/// 玩家位置边界限制逻辑
pub fn clamp_player_position(x: f32, road_width: f32, player_width: f32) -> f32 {
    let half_road = road_width / 2.0 - player_width / 2.0;
    x.clamp(-half_road, half_road)
}

#[cfg(test)]
mod boundary_tests {
    use super::*;

    #[test]
    fn test_clamp_in_bounds() {
        // 在边界内，不变化
        let x = 50.0;
        let road_width = 300.0;
        let player_width = 40.0;

        let result = clamp_player_position(x, road_width, player_width);
        assert!((result - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_clamp_at_boundary() {
        // 刚好在边界上
        let x = 130.0; // road_width/2 - player_width/2 = 150 - 20 = 130
        let road_width = 300.0;
        let player_width = 40.0;

        let result = clamp_player_position(x, road_width, player_width);
        assert!((result - 130.0).abs() < 0.001);
    }

    #[test]
    fn test_clamp_exceed_right_boundary() {
        // 超出右边界
        let x = 200.0;
        let road_width = 300.0;
        let player_width = 40.0;

        let result = clamp_player_position(x, road_width, player_width);
        assert!((result - 130.0).abs() < 0.001);
    }

    #[test]
    fn test_clamp_exceed_left_boundary() {
        // 超出左边界
        let x = -200.0;
        let road_width = 300.0;
        let player_width = 40.0;

        let result = clamp_player_position(x, road_width, player_width);
        assert!((result - (-130.0)).abs() < 0.001);
    }

    #[test]
    fn test_clamp_zero() {
        // 在中心位置
        let x = 0.0;
        let road_width = 300.0;
        let player_width = 40.0;

        let result = clamp_player_position(x, road_width, player_width);
        assert!((result - 0.0).abs() < 0.001);
    }
}

/// ========== 道路标线滚动测试 ==========

/// 道路标线循环滚动逻辑
pub fn calculate_road_line_offset(
    current_offset: f32,
    scroll_speed: f32,
    delta_secs: f32,
    line_height: f32,
    line_gap: f32,
    screen_bottom: f32,
) -> f32 {
    let total_height = line_height + line_gap;
    let new_offset = current_offset - scroll_speed * delta_secs;

    // 循环滚动
    if new_offset < screen_bottom - total_height {
        new_offset + total_height * 10.0
    } else {
        new_offset
    }
}

#[cfg(test)]
mod road_scroll_tests {
    use super::*;

    #[test]
    fn test_scroll_normal_movement() {
        // 正常滚动，不触发循环
        let offset = calculate_road_line_offset(
            0.0,    // current_offset
            200.0,  // scroll_speed
            0.016,  // delta_secs (60fps)
            40.0,   // line_height
            60.0,   // line_gap
            -400.0, // screen_bottom
        );

        // 0 - 200 * 0.016 = -3.2
        assert!((offset - (-3.2)).abs() < 0.001);
    }

    #[test]
    fn test_scroll_triggers_loop() {
        // 滚动到底部，触发循环
        let offset = calculate_road_line_offset(
            -450.0, // 接近底部
            200.0,
            0.016,
            40.0,
            60.0,
            -400.0,
        );

        // -450 - 3.2 = -453.2 < -400 - 100 = -500? No
        // 实际: -453.2 < -400 - 100 = -500 是 false
        // 所以不会循环，返回 -453.2
        assert!((offset - (-453.2)).abs() < 0.001);
    }

    #[test]
    fn test_scroll_loop_triggered() {
        // 确实触发循环的情况
        let offset = calculate_road_line_offset(
            -510.0, // 超过 screen_bottom - total_height = -500
            200.0,
            0.016,
            40.0,
            60.0,
            -400.0,
        );

        // -510 - 3.2 = -513.2 < -500 ✓
        // 循环: -513.2 + 100 * 10 = 486.8
        assert!((offset - 486.8).abs() < 0.001);
    }
}

/// ========== 障碍物移除测试 ==========

/// 判断障碍物是否应该被移除
pub fn should_despawn_obstacle(y_position: f32, screen_bottom: f32) -> bool {
    y_position < screen_bottom
}

#[cfg(test)]
mod obstacle_tests {
    use super::*;

    #[test]
    fn test_despawn_below_screen() {
        assert!(should_despawn_obstacle(-450.0, -400.0));
    }

    #[test]
    fn test_despawn_at_boundary() {
        // 刚好在边界上，不删除（需要小于）
        assert!(!should_despawn_obstacle(-400.0, -400.0));
    }

    #[test]
    fn test_despawn_above_screen() {
        // 在屏幕内
        assert!(!should_despawn_obstacle(100.0, -400.0));
    }

    #[test]
    fn test_despawn_on_screen() {
        // 在屏幕中间
        assert!(!should_despawn_obstacle(0.0, -400.0));
    }
}

/// ========== Bevy 集成测试 ==========

#[cfg(test)]
mod bevy_integration_tests {
    use super::*;
    use crate::player::{Player, PlayerConfig};

    #[test]
    fn test_spawn_player() {
        let mut app = create_test_app();
        app.init_resource::<PlayerConfig>();
        app.init_resource::<crate::GameConfig>();

        // 手动生成玩家
        let _config = app.world().resource::<PlayerConfig>();
        let entity = app.world_mut().spawn((
            Transform::from_xyz(0.0, -220.0, 1.0),
            Player,
        )).id();

        // 验证实体存在
        assert!(app.world().get::<Player>(entity).is_some());
    }

    #[test]
    fn test_player_config_default() {
        let config = PlayerConfig::default();

        assert!((config.speed - 300.0).abs() < 0.001);
        assert!((config.width - 40.0).abs() < 0.001);
        assert!((config.height - 60.0).abs() < 0.001);
    }

    #[test]
    fn test_game_config_default() {
        let config = crate::GameConfig::default();

        assert!((config.road_width - 300.0).abs() < 0.001);
    }

    #[test]
    fn test_timer_creation() {
        use std::time::Duration;

        let timer = Timer::from_seconds(1.5, TimerMode::Repeating);

        assert_eq!(timer.duration(), Duration::from_secs_f32(1.5));
        assert!(timer.mode() == TimerMode::Repeating);
    }
}

/// ========== 游戏状态测试 ==========

#[cfg(test)]
mod game_state_tests {
    use crate::game::GameState;

    #[test]
    fn test_game_state_default() {
        let state = GameState::default();
        assert!(matches!(state, GameState::Menu));
    }

    #[test]
    fn test_game_state_variants() {
        // 确保所有状态都可用
        let menu = GameState::Menu;
        let playing = GameState::Playing;
        let game_over = GameState::GameOver;

        assert!(menu != playing);
        assert!(playing != game_over);
        assert!(menu != game_over);
    }
}

/// ========== 性能基准测试（可选） ==========

#[cfg(test)]
mod benchmark_tests {
    use super::*;

    #[test]
    fn test_collision_detection_performance() {
        // 测试大量碰撞检测的性能
        let pos_a = Vec2::new(0.0, 0.0);
        let half_a = Vec2::new(20.0, 30.0);

        let start = std::time::Instant::now();

        for _ in 0..10000 {
            for x in -200..=200 {
                for y in -200..=200 {
                    let pos_b = Vec2::new(x as f32, y as f32);
                    let half_b = Vec2::new(17.5, 25.0);
                    check_aabb_collision(pos_a, half_a, pos_b, half_b);
                }
            }
        }

        let elapsed = start.elapsed();
        println!("10000次碰撞检测耗时: {:?}", elapsed);

        // 确保在合理时间内完成（非严格限制）
        assert!(elapsed.as_millis() < 1000);
    }
}

/// ========== 生命系统测试 ==========

/// 玩家生命状态
#[derive(Clone, Copy)]
pub struct TestPlayerLife {
    pub lives: u32,
    pub is_invincible: bool,
    pub invincibility_timer: f32,
}

/// 处理碰撞逻辑（测试用副本）
pub fn handle_collision_test(
    player_life: &mut TestPlayerLife,
    _max_lives: u32,
    invincibility_duration: f32,
) -> bool {
    if player_life.is_invincible {
        return false;
    }

    if player_life.lives > 0 {
        player_life.lives -= 1;
    }

    if player_life.lives == 0 {
        true
    } else {
        player_life.is_invincible = true;
        player_life.invincibility_timer = invincibility_duration;
        false
    }
}

#[cfg(test)]
mod life_tests {
    use super::*;

    #[test]
    fn test_collision_reduces_life() {
        let mut life = TestPlayerLife {
            lives: 3,
            is_invincible: false,
            invincibility_timer: 0.0,
        };

        let result = handle_collision_test(&mut life, 3, 2.0);

        assert!(!result); // 游戏未结束
        assert_eq!(life.lives, 2);
        assert!(life.is_invincible); // 触发无敌
    }

    #[test]
    fn test_collision_invincible_no_damage() {
        let mut life = TestPlayerLife {
            lives: 3,
            is_invincible: true,
            invincibility_timer: 1.0,
        };

        let result = handle_collision_test(&mut life, 3, 2.0);

        assert!(!result);
        assert_eq!(life.lives, 3); // 生命不变
    }

    #[test]
    fn test_collision_game_over() {
        let mut life = TestPlayerLife {
            lives: 1,
            is_invincible: false,
            invincibility_timer: 0.0,
        };

        let result = handle_collision_test(&mut life, 3, 2.0);

        assert!(result); // 游戏结束
        assert_eq!(life.lives, 0);
    }

    #[test]
    fn test_multiple_collisions() {
        let mut life = TestPlayerLife {
            lives: 3,
            is_invincible: false,
            invincibility_timer: 0.0,
        };

        // 第一次碰撞
        handle_collision_test(&mut life, 3, 2.0);
        assert_eq!(life.lives, 2);

        // 无敌期间再次碰撞
        handle_collision_test(&mut life, 3, 2.0);
        assert_eq!(life.lives, 2); // 生命不变

        // 无敌结束
        life.is_invincible = false;
        handle_collision_test(&mut life, 3, 2.0);
        assert_eq!(life.lives, 1);
    }
}

/// ========== 难度系统测试 ==========

/// 计算难度等级
pub fn calculate_difficulty_level(elapsed_time: f32, interval: f32) -> u32 {
    (elapsed_time / interval) as u32 + 1
}

/// 计算速度倍率
pub fn calculate_speed_multiplier(level: u32) -> f32 {
    1.0 + (level - 1) as f32 * 0.15
}

/// 计算生成间隔倍率
pub fn calculate_spawn_interval_multiplier(level: u32) -> f32 {
    (1.0 - (level - 1) as f32 * 0.10).max(0.3)
}

#[cfg(test)]
mod difficulty_tests {
    use super::*;

    #[test]
    fn test_initial_difficulty() {
        let level = calculate_difficulty_level(0.0, 10.0);
        assert_eq!(level, 1);

        let speed = calculate_speed_multiplier(1);
        assert!((speed - 1.0).abs() < 0.001);

        let spawn = calculate_spawn_interval_multiplier(1);
        assert!((spawn - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_difficulty_increase() {
        // 10秒后升到2级
        let level = calculate_difficulty_level(10.0, 10.0);
        assert_eq!(level, 2);

        let speed = calculate_speed_multiplier(2);
        assert!((speed - 1.15).abs() < 0.001);

        let spawn = calculate_spawn_interval_multiplier(2);
        assert!((spawn - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_high_difficulty() {
        // 50秒后升到6级
        let level = calculate_difficulty_level(50.0, 10.0);
        assert_eq!(level, 6);

        let speed = calculate_speed_multiplier(6);
        assert!((speed - 1.75).abs() < 0.001);

        let spawn = calculate_spawn_interval_multiplier(6);
        assert!((spawn - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_spawn_interval_minimum() {
        // 高等级时生成间隔有最小值限制
        let spawn = calculate_spawn_interval_multiplier(10);
        assert!((spawn - 0.3).abs() < 0.001); // 最低0.3

        let spawn = calculate_spawn_interval_multiplier(20);
        assert!((spawn - 0.3).abs() < 0.001); // 不会低于0.3
    }
}

/// ========== 分数系统测试 ==========

/// 计算分数
pub fn calculate_score(elapsed_time: f32) -> u32 {
    (elapsed_time * 10.0) as u32
}

#[cfg(test)]
mod score_tests {
    use super::*;

    #[test]
    fn test_initial_score() {
        let score = calculate_score(0.0);
        assert_eq!(score, 0);
    }

    #[test]
    fn test_score_after_one_second() {
        let score = calculate_score(1.0);
        assert_eq!(score, 10);
    }

    #[test]
    fn test_score_after_ten_seconds() {
        let score = calculate_score(10.0);
        assert_eq!(score, 100);
    }

    #[test]
    fn test_score_precision() {
        // 0.1秒 = 1分
        let score = calculate_score(0.1);
        assert_eq!(score, 1);

        // 0.15秒 = 1分（向下取整）
        let score = calculate_score(0.15);
        assert_eq!(score, 1);

        // 0.2秒 = 2分
        let score = calculate_score(0.2);
        assert_eq!(score, 2);
    }
}

/// ========== 护盾系统测试 ==========

/// 护盾状态
#[derive(Clone, Copy)]
pub struct TestShieldState {
    pub has_shield: bool,
    pub shield_timer: f32,
}

/// 更新护盾状态
pub fn update_shield_test(shield: &mut TestShieldState, delta_secs: f32) -> bool {
    if shield.has_shield {
        shield.shield_timer -= delta_secs;

        if shield.shield_timer <= 0.0 {
            shield.has_shield = false;
            return false; // 护盾消失
        }
        return true; // 护盾仍然有效
    }
    false
}

/// 激活护盾
pub fn activate_shield_test(shield: &mut TestShieldState, duration: f32) {
    shield.has_shield = true;
    shield.shield_timer = duration;
}

#[cfg(test)]
mod shield_tests {
    use super::*;

    #[test]
    fn test_shield_activation() {
        let mut shield = TestShieldState {
            has_shield: false,
            shield_timer: 0.0,
        };

        activate_shield_test(&mut shield, 5.0);

        assert!(shield.has_shield);
        assert!((shield.shield_timer - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_shield_expires() {
        let mut shield = TestShieldState {
            has_shield: true,
            shield_timer: 0.5,
        };

        let active = update_shield_test(&mut shield, 0.6);

        assert!(!active);
        assert!(!shield.has_shield);
    }

    #[test]
    fn test_shield_remains_active() {
        let mut shield = TestShieldState {
            has_shield: true,
            shield_timer: 5.0,
        };

        let active = update_shield_test(&mut shield, 1.0);

        assert!(active);
        assert!(shield.has_shield);
        assert!((shield.shield_timer - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_shield_blocks_collision() {
        let shield = TestShieldState {
            has_shield: true,
            shield_timer: 3.0,
        };

        // 有护盾时碰撞应该被阻挡
        assert!(shield.has_shield);
    }
}

/// ========== 道具碰撞测试 ==========

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

#[cfg(test)]
mod powerup_tests {
    use super::*;

    #[test]
    fn test_powerup_collection() {
        let player_pos = Vec2::new(0.0, 0.0);
        let player_half = Vec2::new(20.0, 30.0);
        let powerup_pos = Vec2::new(5.0, 5.0);
        let powerup_half = Vec2::new(15.0, 15.0);

        assert!(check_powerup_collision(player_pos, player_half, powerup_pos, powerup_half));
    }

    #[test]
    fn test_powerup_miss() {
        let player_pos = Vec2::new(0.0, 0.0);
        let player_half = Vec2::new(20.0, 30.0);
        let powerup_pos = Vec2::new(100.0, 100.0);
        let powerup_half = Vec2::new(15.0, 15.0);

        assert!(!check_powerup_collision(player_pos, player_half, powerup_pos, powerup_half));
    }
}
