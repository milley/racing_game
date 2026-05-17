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
    use crate::game::GameState;

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
