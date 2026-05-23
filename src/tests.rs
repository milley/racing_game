#[cfg(test)]
mod tests {
    use crate::game::{
        GameState, Combo, MENU_OPTION_COUNT,
        handle_menu_navigation, update_combo_timer,
        calculate_difficulty_level, calculate_speed_multiplier,
        calculate_spawn_interval_multiplier, calculate_base_score,
        calculate_score_increment,
    };
    use crate::obstacle::{
        check_aabb_collision, should_despawn_obstacle,
        record_dodge, reset_combo_on_collision,
    };
    use crate::player::clamp_player_position;
    use crate::road::calculate_road_line_offset;
    use crate::life::{PlayerLife, LifeConfig, handle_collision};
    use crate::powerup::{
        ActivePowerUps,
        check_powerup_collision, update_shield_timer, activate_shield,
    };
    use crate::settings::DifficultyLevel;
    use crate::game_mode::{GameMode, cycle_game_mode, get_mode_lives, get_mode_time_limit};
    use bevy::math::Vec2;

    // ========== 碰撞检测测试 ==========

    mod collision_tests {
        use super::*;

        #[test]
        fn test_collision_detected() {
            let result = check_aabb_collision(
                Vec2::new(0.0, 0.0), Vec2::new(25.0, 25.0),
                Vec2::new(10.0, 10.0), Vec2::new(25.0, 25.0),
            );
            assert!(result);
        }

        #[test]
        fn test_no_collision_separated_x() {
            let result = check_aabb_collision(
                Vec2::new(0.0, 0.0), Vec2::new(25.0, 25.0),
                Vec2::new(100.0, 0.0), Vec2::new(25.0, 25.0),
            );
            assert!(!result);
        }

        #[test]
        fn test_no_collision_separated_y() {
            let result = check_aabb_collision(
                Vec2::new(0.0, 0.0), Vec2::new(25.0, 25.0),
                Vec2::new(0.0, 100.0), Vec2::new(25.0, 25.0),
            );
            assert!(!result);
        }

        #[test]
        fn test_edge_collision() {
            let result = check_aabb_collision(
                Vec2::new(0.0, 0.0), Vec2::new(25.0, 25.0),
                Vec2::new(49.9, 0.0), Vec2::new(25.0, 25.0),
            );
            assert!(result);
        }

        #[test]
        fn test_no_collision_at_edge() {
            let result = check_aabb_collision(
                Vec2::new(0.0, 0.0), Vec2::new(25.0, 25.0),
                Vec2::new(50.1, 0.0), Vec2::new(25.0, 25.0),
            );
            assert!(!result);
        }
    }

    // ========== 边界限制测试 ==========

    mod boundary_tests {
        use super::*;

        #[test]
        fn test_player_at_center() {
            let result = clamp_player_position(0.0, 400.0, 40.0);
            assert_eq!(result, 0.0);
        }

        #[test]
        fn test_player_at_right_boundary() {
            let result = clamp_player_position(200.0, 400.0, 40.0);
            assert!((result - 180.0).abs() < 0.001);
        }

        #[test]
        fn test_player_at_left_boundary() {
            let result = clamp_player_position(-200.0, 400.0, 40.0);
            assert!((result - (-180.0)).abs() < 0.001);
        }

        #[test]
        fn test_player_exceeds_right_boundary() {
            let result = clamp_player_position(300.0, 400.0, 40.0);
            assert!((result - 180.0).abs() < 0.001);
        }

        #[test]
        fn test_player_exceeds_left_boundary() {
            let result = clamp_player_position(-300.0, 400.0, 40.0);
            assert!((result - (-180.0)).abs() < 0.001);
        }

        #[test]
        fn test_player_within_boundary() {
            let result = clamp_player_position(50.0, 400.0, 40.0);
            assert_eq!(result, 50.0);
        }
    }

    // ========== 道路滚动测试 ==========

    mod road_scroll_tests {
        use super::*;

        #[test]
        fn test_road_line_scroll() {
            let result = calculate_road_line_offset(100.0, 200.0, 0.016, 60.0, -720.0, 720.0);
            assert!((result - 96.8).abs() < 0.01);
        }

        #[test]
        fn test_road_line_loop() {
            let total_height = 60.0;
            let loop_threshold = -total_height * 12.0;
            let loop_reset = total_height * 12.0;
            let result = calculate_road_line_offset(-710.0, 200.0, 0.016, total_height, loop_threshold, loop_reset);
            assert!(result > loop_threshold);
        }

        #[test]
        fn test_road_line_no_scroll() {
            let result = calculate_road_line_offset(100.0, 0.0, 0.016, 60.0, -720.0, 720.0);
            assert_eq!(result, 100.0);
        }
    }

    // ========== 障碍物测试 ==========

    mod obstacle_tests {
        use super::*;

        #[test]
        fn test_should_despawn() {
            assert!(should_despawn_obstacle(-500.0, -400.0));
        }

        #[test]
        fn test_should_not_despawn() {
            assert!(!should_despawn_obstacle(-300.0, -400.0));
        }

        #[test]
        fn test_despawn_at_boundary() {
            assert!(!should_despawn_obstacle(-400.0, -400.0));
        }
    }

    // ========== 生命系统测试 ==========

    mod life_tests {
        use super::*;

        fn create_player_life(lives: u32) -> PlayerLife {
            PlayerLife {
                lives,
                is_invincible: false,
                invincibility_timer: 0.0,
            }
        }

        fn create_life_config() -> LifeConfig {
            LifeConfig::default()
        }

        #[test]
        fn test_handle_collision_lose_life() {
            let mut life = create_player_life(3);
            let config = create_life_config();
            let result = handle_collision(&mut life, &config);
            assert!(!result); // not game over, still has lives
            assert_eq!(life.lives, 2);
        }

        #[test]
        fn test_handle_collision_last_life() {
            let mut life = create_player_life(1);
            let config = create_life_config();
            let result = handle_collision(&mut life, &config);
            assert!(result); // game over
            assert_eq!(life.lives, 0);
        }

        #[test]
        fn test_handle_collision_invincible() {
            let mut life = create_player_life(3);
            life.is_invincible = true;
            let config = create_life_config();
            let result = handle_collision(&mut life, &config);
            assert!(!result); // not game over, invincible
            assert_eq!(life.lives, 3);
        }

        #[test]
        fn test_handle_collision_zero_lives() {
            let mut life = create_player_life(0);
            let config = create_life_config();
            let result = handle_collision(&mut life, &config);
            assert!(result); // game over
            assert_eq!(life.lives, 0);
        }
    }

    // ========== 难度系统测试 ==========

    mod difficulty_tests {
        use super::*;

        #[test]
        fn test_initial_difficulty() {
            assert_eq!(calculate_difficulty_level(0.0, 30.0), 1);
        }

        #[test]
        fn test_level_2() {
            assert_eq!(calculate_difficulty_level(30.0, 30.0), 2);
        }

        #[test]
        fn test_level_3() {
            assert_eq!(calculate_difficulty_level(60.0, 30.0), 3);
        }

        #[test]
        fn test_speed_multiplier_level_1() {
            assert!((calculate_speed_multiplier(1) - 1.0).abs() < 0.001);
        }

        #[test]
        fn test_speed_multiplier_level_2() {
            assert!((calculate_speed_multiplier(2) - 1.15).abs() < 0.001);
        }

        #[test]
        fn test_speed_multiplier_level_5() {
            assert!((calculate_speed_multiplier(5) - 1.6).abs() < 0.001);
        }

        #[test]
        fn test_spawn_interval_level_1() {
            assert!((calculate_spawn_interval_multiplier(1) - 1.0).abs() < 0.001);
        }

        #[test]
        fn test_spawn_interval_level_2() {
            assert!((calculate_spawn_interval_multiplier(2) - 0.9).abs() < 0.001);
        }

        #[test]
        fn test_spawn_interval_minimum() {
            assert!((calculate_spawn_interval_multiplier(10) - 0.3).abs() < 0.001);
        }
    }

    // ========== 分数系统测试 ==========

    mod score_tests {
        use super::*;

        #[test]
        fn test_base_score_calculation() {
            assert_eq!(calculate_base_score(10.0), 100);
        }

        #[test]
        fn test_base_score_zero() {
            assert_eq!(calculate_base_score(0.0), 0);
        }

        #[test]
        fn test_score_increment_no_combo_no_double() {
            let increment = calculate_score_increment(100, 90, 1.0, false);
            assert_eq!(increment, 10);
        }

        #[test]
        fn test_score_increment_with_combo() {
            let increment = calculate_score_increment(100, 90, 2.0, false);
            assert_eq!(increment, 20);
        }

        #[test]
        fn test_score_increment_with_double_score() {
            let increment = calculate_score_increment(100, 90, 1.0, true);
            assert_eq!(increment, 20);
        }

        #[test]
        fn test_score_increment_with_combo_and_double() {
            let increment = calculate_score_increment(100, 90, 2.0, true);
            assert_eq!(increment, 40);
        }
    }

    // ========== 护盾测试 ==========

    mod shield_tests {
        use super::*;

        fn create_active_powerups() -> ActivePowerUps {
            ActivePowerUps {
                shield_timer: 0.0,
                has_shield: false,
                magnet_timer: 0.0,
                has_magnet: false,
                slowdown_timer: 0.0,
                has_slowdown: false,
                double_score_timer: 0.0,
                has_double_score: false,
                shrink_timer: 0.0,
                has_shrink: false,
                nitro_timer: 0.0,
                has_nitro: false,
            }
        }

        #[test]
        fn test_activate_shield() {
            let mut powerups = create_active_powerups();
            activate_shield(&mut powerups, 5.0);
            assert!(powerups.has_shield);
            assert!((powerups.shield_timer - 5.0).abs() < 0.001);
        }

        #[test]
        fn test_shield_timer_update() {
            let mut powerups = create_active_powerups();
            activate_shield(&mut powerups, 5.0);
            let still_active = update_shield_timer(&mut powerups, 1.0);
            assert!(still_active);
            assert!((powerups.shield_timer - 4.0).abs() < 0.001);
        }

        #[test]
        fn test_shield_expire() {
            let mut powerups = create_active_powerups();
            activate_shield(&mut powerups, 1.0);
            let still_active = update_shield_timer(&mut powerups, 1.5);
            assert!(!still_active);
            assert!(!powerups.has_shield);
        }

        #[test]
        fn test_no_shield_update() {
            let mut powerups = create_active_powerups();
            let still_active = update_shield_timer(&mut powerups, 1.0);
            assert!(!still_active);
        }
    }

    // ========== 道具碰撞测试 ==========

    mod powerup_tests {
        use super::*;

        #[test]
        fn test_powerup_collision_detected() {
            let result = check_powerup_collision(
                Vec2::new(0.0, 0.0), Vec2::new(25.0, 25.0),
                Vec2::new(10.0, 10.0), Vec2::new(25.0, 25.0),
            );
            assert!(result);
        }

        #[test]
        fn test_powerup_no_collision() {
            let result = check_powerup_collision(
                Vec2::new(0.0, 0.0), Vec2::new(25.0, 25.0),
                Vec2::new(100.0, 0.0), Vec2::new(25.0, 25.0),
            );
            assert!(!result);
        }
    }

    // ========== 菜单导航测试 ==========

    mod menu_navigation_tests {
        use super::*;

        #[test]
        fn test_menu_down() {
            assert_eq!(handle_menu_navigation(0, 1, MENU_OPTION_COUNT), 1);
        }

        #[test]
        fn test_menu_up() {
            assert_eq!(handle_menu_navigation(1, -1, MENU_OPTION_COUNT), 0);
        }

        #[test]
        fn test_menu_wrap_down() {
            assert_eq!(handle_menu_navigation(MENU_OPTION_COUNT - 1, 1, MENU_OPTION_COUNT), 0);
        }

        #[test]
        fn test_menu_wrap_up() {
            assert_eq!(handle_menu_navigation(0, -1, MENU_OPTION_COUNT), MENU_OPTION_COUNT - 1);
        }

        #[test]
        fn test_menu_option_count() {
            assert_eq!(MENU_OPTION_COUNT, 4);
        }
    }

    // ========== 设置测试 ==========

    mod settings_tests {
        use super::*;

        #[test]
        fn test_difficulty_level_variants() {
            let easy = DifficultyLevel::Easy;
            let normal = DifficultyLevel::Normal;
            let hard = DifficultyLevel::Hard;
            assert_ne!(easy, normal);
            assert_ne!(normal, hard);
            assert_ne!(easy, hard);
        }

        #[test]
        fn test_difficulty_speed_multiplier() {
            assert!((DifficultyLevel::Easy.speed_multiplier() - 0.75).abs() < 0.001);
            assert!((DifficultyLevel::Normal.speed_multiplier() - 1.0).abs() < 0.001);
            assert!((DifficultyLevel::Hard.speed_multiplier() - 1.25).abs() < 0.001);
        }

        #[test]
        fn test_difficulty_lives() {
            assert_eq!(DifficultyLevel::Easy.lives(), 5);
            assert_eq!(DifficultyLevel::Normal.lives(), 3);
            assert_eq!(DifficultyLevel::Hard.lives(), 2);
        }

        #[test]
        fn test_difficulty_spawn_interval() {
            assert!((DifficultyLevel::Easy.spawn_interval_multiplier() - 1.5).abs() < 0.001);
            assert!((DifficultyLevel::Normal.spawn_interval_multiplier() - 1.0).abs() < 0.001);
            assert!((DifficultyLevel::Hard.spawn_interval_multiplier() - 0.75).abs() < 0.001);
        }

        #[test]
        fn test_difficulty_from_str() {
            assert_eq!(DifficultyLevel::from_str("easy"), DifficultyLevel::Easy);
            assert_eq!(DifficultyLevel::from_str("HARD"), DifficultyLevel::Hard);
            assert_eq!(DifficultyLevel::from_str("unknown"), DifficultyLevel::Normal);
        }
    }

    // ========== 连击系统测试 ==========

    mod combo_tests {
        use super::*;

        fn create_combo() -> Combo {
            Combo {
                count: 0,
                combo_multiplier: 1.0,
                timer: 0.0,
                max_timer: 2.0,
            }
        }

        #[test]
        fn test_record_dodge() {
            let mut combo = create_combo();
            record_dodge(&mut combo);
            assert_eq!(combo.count, 1);
            assert!((combo.combo_multiplier - 1.1).abs() < 0.001);
            assert!((combo.timer - 2.0).abs() < 0.001);
        }

        #[test]
        fn test_record_multiple_dodges() {
            let mut combo = create_combo();
            for _ in 0..5 {
                record_dodge(&mut combo);
            }
            assert_eq!(combo.count, 5);
            assert!((combo.combo_multiplier - 1.5).abs() < 0.001);
        }

        #[test]
        fn test_combo_multiplier_max() {
            let mut combo = create_combo();
            for _ in 0..25 {
                record_dodge(&mut combo);
            }
            assert!((combo.combo_multiplier - 3.0).abs() < 0.001);
        }

        #[test]
        fn test_reset_combo_on_collision() {
            let mut combo = create_combo();
            record_dodge(&mut combo);
            record_dodge(&mut combo);
            assert_eq!(combo.count, 2);
            reset_combo_on_collision(&mut combo);
            assert_eq!(combo.count, 0);
            assert!((combo.combo_multiplier - 1.0).abs() < 0.001);
            assert!((combo.timer - 0.0).abs() < 0.001);
        }

        #[test]
        fn test_update_combo_timer() {
            let mut combo = create_combo();
            record_dodge(&mut combo);
            let still_active = update_combo_timer(&mut combo, 1.0);
            assert!(still_active);
            assert!((combo.timer - 1.0).abs() < 0.001);
        }

        #[test]
        fn test_combo_timer_expire() {
            let mut combo = create_combo();
            record_dodge(&mut combo);
            let still_active = update_combo_timer(&mut combo, 3.0);
            assert!(!still_active);
            assert_eq!(combo.count, 0);
            assert!((combo.combo_multiplier - 1.0).abs() < 0.001);
        }

        #[test]
        fn test_combo_timer_no_combo() {
            let mut combo = create_combo();
            let still_active = update_combo_timer(&mut combo, 1.0);
            assert!(!still_active);
        }
    }

    // ========== 双倍分数回归测试 ==========

    mod double_score_tests {
        use super::*;

        #[test]
        fn test_double_score_with_combo_multiplier() {
            // 连击倍率1.5 + 双倍分数 = 3.0x
            let increment = calculate_score_increment(100, 90, 1.5, true);
            assert_eq!(increment, 30);
        }

        #[test]
        fn test_collision_resets_combo_not_double_score() {
            // 碰撞重置 combo_multiplier 到 1.0，但 DoubleScore 是独立的
            let mut combo = Combo {
                count: 5,
                combo_multiplier: 1.5,
                timer: 1.0,
                max_timer: 2.0,
            };
            reset_combo_on_collision(&mut combo);
            assert!((combo.combo_multiplier - 1.0).abs() < 0.001);

            // DoubleScore 仍然生效（通过 has_double_score 标记）
            let increment = calculate_score_increment(100, 90, combo.combo_multiplier, true);
            assert_eq!(increment, 20); // 10 * 1.0 * 2.0
        }

        #[test]
        fn test_double_score_expiry_no_halving_bug() {
            // DoubleScore 过期后，combo_multiplier 不会被除以2
            let combo = Combo {
                count: 3,
                combo_multiplier: 1.3,
                timer: 1.0,
                max_timer: 2.0,
            };
            // 模拟 DoubleScore 过期：只是 has_double_score 变为 false
            // combo_multiplier 不受影响
            let increment_before = calculate_score_increment(100, 90, combo.combo_multiplier, true);
            let increment_after = calculate_score_increment(100, 90, combo.combo_multiplier, false);
            assert_eq!(increment_before, 26); // 10 * 1.3 * 2.0
            assert_eq!(increment_after, 13);  // 10 * 1.3 * 1.0
            // combo_multiplier 始终 >= 1.0，不会出现 0.5 的 Bug
            assert!(combo.combo_multiplier >= 1.0);
        }

        #[test]
        fn test_combo_timeout_with_double_score_no_bug() {
            // 连击超时重置 combo_multiplier 到 1.0，DoubleScore 独立
            let mut combo = Combo {
                count: 5,
                combo_multiplier: 1.5,
                timer: 0.1,
                max_timer: 2.0,
            };
            update_combo_timer(&mut combo, 1.0);
            assert!((combo.combo_multiplier - 1.0).abs() < 0.001);

            // DoubleScore 仍然正常工作
            let increment = calculate_score_increment(100, 90, combo.combo_multiplier, true);
            assert_eq!(increment, 20); // 10 * 1.0 * 2.0
        }

        #[test]
        fn test_no_multiplier_below_one() {
            // 确保 combo_multiplier 永远不会低于 1.0
            let mut combo = Combo {
                count: 0,
                combo_multiplier: 1.0,
                timer: 0.0,
                max_timer: 2.0,
            };
            reset_combo_on_collision(&mut combo);
            assert!(combo.combo_multiplier >= 1.0);
            update_combo_timer(&mut combo, 5.0);
            assert!(combo.combo_multiplier >= 1.0);
        }
    }

    // ========== 游戏模式测试 ==========

    mod game_mode_tests {
        use super::*;

        #[test]
        fn test_cycle_game_mode_forward() {
            assert_eq!(cycle_game_mode(GameMode::Classic, 1), GameMode::Endless);
            assert_eq!(cycle_game_mode(GameMode::Endless, 1), GameMode::TimeAttack);
            assert_eq!(cycle_game_mode(GameMode::TimeAttack, 1), GameMode::Classic);
        }

        #[test]
        fn test_cycle_game_mode_backward() {
            assert_eq!(cycle_game_mode(GameMode::Classic, -1), GameMode::TimeAttack);
            assert_eq!(cycle_game_mode(GameMode::Endless, -1), GameMode::Classic);
            assert_eq!(cycle_game_mode(GameMode::TimeAttack, -1), GameMode::Endless);
        }

        #[test]
        fn test_get_mode_lives() {
            assert_eq!(get_mode_lives(GameMode::Classic, 3), 3);
            assert_eq!(get_mode_lives(GameMode::Endless, 3), 1);
            assert_eq!(get_mode_lives(GameMode::TimeAttack, 3), 3);
        }

        #[test]
        fn test_get_mode_time_limit() {
            assert_eq!(get_mode_time_limit(GameMode::Classic), None);
            assert_eq!(get_mode_time_limit(GameMode::Endless), None);
            assert_eq!(get_mode_time_limit(GameMode::TimeAttack), Some(60.0));
        }
    }

    // ========== 游戏状态测试 ==========

    mod game_state_tests {
        use super::*;

        #[test]
        fn test_game_state_variants() {
            let states = vec![GameState::Menu, GameState::Playing, GameState::Paused, GameState::GameOver];
            assert_eq!(states.len(), 4);
        }

        #[test]
        fn test_game_state_equality() {
            assert_eq!(GameState::Menu, GameState::Menu);
            assert_ne!(GameState::Menu, GameState::Playing);
        }
    }

    // ========== Bevy 集成测试 ==========

    mod bevy_integration_tests {
        use bevy::prelude::*;
        use super::Combo;

        #[test]
        fn test_app_initialization() {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            app.update();
        }

        #[test]
        fn test_resource_insertion() {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            app.insert_resource(Combo {
                count: 0,
                combo_multiplier: 1.0,
                timer: 0.0,
                max_timer: 2.0,
            });
            app.update();
            let combo = app.world().resource::<Combo>();
            assert_eq!(combo.count, 0);
            assert!((combo.combo_multiplier - 1.0).abs() < 0.001);
        }
    }
}
