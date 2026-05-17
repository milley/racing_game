use bevy::prelude::*;
use game::GamePlugin;

mod game;
mod player;
mod road;
mod obstacle;

fn main() {
    App::new()
        // 窗口配置
        .init_resource::<WindowConfig>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Retro Racing".into(),
                resolution: (400.0, 600.0).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        // 插入游戏资源
        .init_resource::<GameConfig>()
        // 添加游戏插件
        .add_plugins(GamePlugin)
        .run();
}

/// 窗口配置
#[derive(Resource, Default)]
struct WindowConfig;

/// 游戏配置
#[derive(Resource)]
struct GameConfig {
    /// 道路宽度
    road_width: f32,
    /// 道路边距
    road_margin: f32,
    /// 游戏速度
    base_speed: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            road_width: 300.0,
            road_margin: 50.0,
            base_speed: 200.0,
        }
    }
}
