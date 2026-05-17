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
            // 添加系统
            .add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(OnEnter(GameState::Playing), setup_game)
            .add_systems(OnExit(GameState::Playing), cleanup_game)
            .add_systems(Update, menu_system.run_if(in_state(GameState::Menu)));
    }
}

/// 菜单设置
fn setup_menu(mut commands: Commands) {
    commands.spawn(Camera2d);

    // 标题文字
    commands.spawn((
        Text::new("RETRO RACING"),
        TextFont {
            font_size: 48.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(35.0),
            left: Val::Percent(50.0),
            ..default()
        },
    ));

    // 提示文字
    commands.spawn((
        Text::new("Press SPACE to Start"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(0.8, 0.8, 0.8)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(55.0),
            left: Val::Percent(50.0),
            ..default()
        },
    ));
}

/// 游戏设置
fn setup_game(mut commands: Commands) {
    // 游戏相机
    commands.spawn(Camera2d);

    // 分数显示
    commands.spawn((
        Text::new("Score: 0"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        ScoreText,
    ));
}

/// 清理游戏
fn cleanup_game(mut commands: Commands, query: Query<Entity, With<GameEntity>>) {
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

/// 分数文本标记
#[derive(Component)]
pub struct ScoreText;
