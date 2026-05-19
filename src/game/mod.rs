use bevy::prelude::*;

use crate::{
    obstacle::ObstaclePlugin,
    player::PlayerPlugin,
    road::RoadPlugin,
    particle::ParticlePlugin,
    life::LifePlugin,
    powerup::PowerUpPlugin,
    audio::AudioPlugin,
    graphics::PixelGraphicsPlugin,
};

/// 游戏状态
#[derive(States, Default, Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

/// 分数资源
#[derive(Resource)]
pub struct Score {
    /// 当前分数
    pub value: u32,
    /// 最高分
    pub high_score: u32,
}

impl Default for Score {
    fn default() -> Self {
        Self {
            value: 0,
            high_score: 0,
        }
    }
}

/// 难度资源
#[derive(Resource)]
pub struct Difficulty {
    /// 当前难度等级
    pub level: u32,
    /// 速度倍率
    pub speed_multiplier: f32,
    /// 障碍物生成间隔倍率
    pub spawn_interval_multiplier: f32,
}

impl Default for Difficulty {
    fn default() -> Self {
        Self {
            level: 1,
            speed_multiplier: 1.0,
            spawn_interval_multiplier: 1.0,
        }
    }
}

/// 游戏主插件
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // 初始化游戏状态
            .init_state::<GameState>()
            // 初始化资源和计时器
            .init_resource::<Score>()
            .init_resource::<Difficulty>()
            .init_resource::<GameTimer>()
            // 添加子插件
            .add_plugins((
                PlayerPlugin,
                RoadPlugin,
                ObstaclePlugin,
                ParticlePlugin,
                LifePlugin,
                PowerUpPlugin,
                AudioPlugin,
                PixelGraphicsPlugin,
            ))
            // 启动时生成相机（只生成一次）
            .add_systems(Startup, setup_camera)
            // 添加系统
            .add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(OnExit(GameState::Menu), cleanup_menu)
            .add_systems(OnEnter(GameState::Playing), setup_game)
            .add_systems(OnExit(GameState::Playing), cleanup_game)
            .add_systems(OnEnter(GameState::GameOver), setup_game_over)
            .add_systems(OnExit(GameState::GameOver), cleanup_game_over)
            .add_systems(Update, (
                menu_system.run_if(in_state(GameState::Menu)),
                update_score.run_if(in_state(GameState::Playing)),
                update_difficulty.run_if(in_state(GameState::Playing)),
                game_over_system.run_if(in_state(GameState::GameOver)),
            ));
    }
}

/// 游戏计时器
#[derive(Resource)]
pub struct GameTimer {
    /// 游戏时间
    pub elapsed: f32,
    /// 上次难度提升时间
    pub last_difficulty_increase: f32,
}

impl Default for GameTimer {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            last_difficulty_increase: 0.0,
        }
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
                Text::new("Arrow Keys or A/D to Move"),
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
fn setup_game(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut difficulty: ResMut<Difficulty>,
    mut game_timer: ResMut<GameTimer>,
) {
    // 重置游戏状态
    score.value = 0;
    difficulty.level = 1;
    difficulty.speed_multiplier = 1.0;
    difficulty.spawn_interval_multiplier = 1.0;
    game_timer.elapsed = 0.0;
    game_timer.last_difficulty_increase = 0.0;

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
            GameEntity,
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
    // 只删除父实体，子实体会自动被递归删除
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

/// 游戏结束界面
fn setup_game_over(mut commands: Commands, mut score: ResMut<Score>) {
    // 更新最高分
    if score.value > score.high_score {
        score.high_score = score.value;
    }

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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
            GameOverUI,
        ))
        .with_children(|parent| {
            // GAME OVER 标题
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // 最终分数
            let score_text = format!("Score: {}", score.value);
            parent.spawn((
                Text::new(score_text),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // 最高分
            let high_score_text = format!("Best: {}", score.high_score);
            parent.spawn((
                Text::new(high_score_text),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // 重新开始提示
            parent.spawn((
                Text::new("Press SPACE to Restart"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node::default(),
            ));

            // 返回菜单提示
            parent.spawn((
                Text::new("Press ESC for Menu"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
            ));
        });
}

/// 清理游戏结束界面
fn cleanup_game_over(mut commands: Commands, query: Query<Entity, With<GameOverUI>>) {
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

/// 游戏结束系统
fn game_over_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Menu);
    }
}

/// 更新分数
fn update_score(
    mut score: ResMut<Score>,
    mut game_timer: ResMut<GameTimer>,
    time: Res<Time>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    game_timer.elapsed += time.delta_secs();

    // 每0.1秒加1分
    let new_score = (game_timer.elapsed * 10.0) as u32;
    if new_score != score.value {
        score.value = new_score;

        // 更新UI
        if let Ok(mut text) = query.single_mut() {
            **text = format!("Score: {}", score.value);
        }
    }
}

/// 更新难度
fn update_difficulty(
    mut difficulty: ResMut<Difficulty>,
    game_timer: Res<GameTimer>,
) {
    // 每10秒提升一级难度
    let new_level = (game_timer.elapsed / 10.0) as u32 + 1;
    if new_level != difficulty.level {
        difficulty.level = new_level;
        // 速度每级增加15%
        difficulty.speed_multiplier = 1.0 + (difficulty.level - 1) as f32 * 0.15;
        // 生成间隔每级减少10%（最低0.3倍）
        difficulty.spawn_interval_multiplier = (1.0 - (difficulty.level - 1) as f32 * 0.10).max(0.3);
    }
}

/// 游戏实体标记
#[derive(Component)]
pub struct GameEntity;

/// 游戏UI标记
#[derive(Component)]
struct GameUI;

/// 游戏结束UI标记
#[derive(Component)]
struct GameOverUI;

/// 分数文本标记
#[derive(Component)]
pub struct ScoreText;
