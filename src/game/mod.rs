use bevy::prelude::*;

use crate::{obstacle::ObstaclePlugin, player::PlayerPlugin, road::RoadPlugin};

/// 游戏状态
#[derive(States, Default, Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

/// 游戏主插件
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // 初始化游戏状态
            .init_state::<GameState>()
            // 添加子插件
            .add_plugins((PlayerPlugin, RoadPlugin, ObstaclePlugin))
            // 启动时生成相机（只生成一次）
            .add_systems(Startup, setup_camera)
            // 添加系统
            .add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(OnExit(GameState::Menu), cleanup_menu)
            .add_systems(OnEnter(GameState::Playing), setup_game)
            .add_systems(OnExit(GameState::Playing), cleanup_game)
            .add_systems(Update, menu_system.run_if(in_state(GameState::Menu)));
    }
}

/// 生成相机（全局唯一）
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// 菜单设置
fn setup_menu(mut commands: Commands) {

    // 根容器 - 全屏居中
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
        ))
        .with_children(|parent| {
            // 标题文字
            parent.spawn((
                Text::new("RETRO RACING"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // 提示文字
            parent.spawn((
                Text::new("Press SPACE to Start"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node::default(),
            ));

            // 操作说明
            parent.spawn((
                Text::new("← → or A D to Move"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            ));
        });
}

/// 游戏设置
fn setup_game(mut commands: Commands) {
    // UI 根容器
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            GameUI,
        ))
        .with_children(|parent| {
            // 分数显示
            parent.spawn((
                Text::new("Score: 0"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Node {
                    margin: UiRect::px(10.0, 10.0, 10.0, 0.0),
                    ..default()
                },
                ScoreText,
            ));
        });
}

/// 清理游戏
fn cleanup_game(mut commands: Commands, query: Query<Entity, With<GameEntity>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 清理菜单
fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<Node>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 菜单系统
fn menu_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
}

/// 游戏实体标记
#[derive(Component)]
pub struct GameEntity;

/// 游戏UI标记
#[derive(Component)]
struct GameUI;

/// 分数文本标记
#[derive(Component)]
pub struct ScoreText;
