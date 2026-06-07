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
    save::SavePlugin,
    settings::SettingsPlugin,
    achievement::AchievementPlugin,
    game_mode::GameModePlugin,
};

/// 游戏状态
#[derive(States, Default, Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    Menu,
    Settings,
    Achievements,
    Help,
    Playing,
    Paused,
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
            .init_resource::<Combo>()
            // 添加子插件
            .add_plugins((
                SavePlugin,
                SettingsPlugin,
                AchievementPlugin,
                GameModePlugin,
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
            .add_systems(OnEnter(GameState::Paused), setup_pause_menu)
            .add_systems(OnExit(GameState::Paused), cleanup_pause_menu)
            .add_systems(OnEnter(GameState::GameOver), setup_game_over)
            .add_systems(OnExit(GameState::GameOver), (cleanup_game_over, crate::save::save_game_data))
            .add_systems(OnEnter(GameState::Help), setup_help)
            .add_systems(OnExit(GameState::Help), cleanup_help)
            .add_systems(Update, help_system.run_if(in_state(GameState::Help)))
            .add_systems(Update, (
                menu_system.run_if(in_state(GameState::Menu)),
                pause_system.run_if(in_state(GameState::Playing)),
                resume_system.run_if(in_state(GameState::Paused)),
                update_score.run_if(in_state(GameState::Playing)),
                update_difficulty.run_if(in_state(GameState::Playing)),
                update_combo.run_if(in_state(GameState::Playing)),
                game_over_system.run_if(in_state(GameState::GameOver)),
            ));
    }
}

/// 连击系统
#[derive(Resource)]
pub struct Combo {
    /// 连击计数
    pub count: u32,
    /// 分数倍率（仅由连击驱动）
    pub combo_multiplier: f32,
    /// 连击计时器
    pub timer: f32,
    /// 连击最大时间
    pub max_timer: f32,
}

impl Default for Combo {
    fn default() -> Self {
        Self {
            count: 0,
            combo_multiplier: 1.0,
            timer: 0.0,
            max_timer: 2.0,
        }
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

/// 菜单选项数量
pub const MENU_OPTION_COUNT: usize = 5;

/// 处理菜单上下导航
pub fn handle_menu_navigation(current: usize, direction: i32, option_count: usize) -> usize {
    if direction < 0 {
        if current == 0 { option_count - 1 } else { current - 1 }
    } else {
        (current + 1) % option_count
    }
}

/// 更新连击计时器，返回是否仍然有效
pub fn update_combo_timer(combo: &mut Combo, delta_secs: f32) -> bool {
    if combo.count > 0 {
        combo.timer -= delta_secs;
        if combo.timer <= 0.0 {
            combo.count = 0;
            combo.combo_multiplier = 1.0;
            combo.timer = 0.0;
            return false;
        }
        return true;
    }
    false
}

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

/// 计算基础分数
pub fn calculate_base_score(elapsed_time: f32) -> u32 {
    (elapsed_time * 10.0) as u32
}

/// 计算分数增量（含连击倍率和双倍分数）
pub fn calculate_score_increment(
    base_score: u32,
    current_score: u32,
    combo_multiplier: f32,
    has_double_score: bool,
) -> u32 {
    let double_score_factor = if has_double_score { 2.0 } else { 1.0 };
    let increment = (base_score - current_score) as f32 * combo_multiplier * double_score_factor;
    increment as u32
}

/// 当前选中的菜单选项
#[derive(Resource, Default)]
struct MenuSelection(usize);

/// 菜单选项标记
#[derive(Component)]
enum MenuOption {
    Start,
    GameMode,
    HowToPlay,
    Settings,
    Achievements,
}

/// 菜单设置
fn setup_menu(
    mut commands: Commands,
    current_mode: Res<crate::game_mode::CurrentGameMode>,
) {
    commands.insert_resource(MenuSelection(0));

    let mode_name = crate::game_mode::get_mode_name(current_mode.mode);

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
            MenuUI,
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

            // 开始游戏
            parent.spawn((
                Text::new("> Start Game <"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
                MenuOption::Start,
            ));

            // 游戏模式
            parent.spawn((
                Text::new(format!("  Mode: {}  ", mode_name)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
                MenuOption::GameMode,
            ));

            // 设置
            parent.spawn((
                Text::new("  Settings  "),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
                MenuOption::Settings,
            ));

            // 成就
            parent.spawn((
                Text::new("  Achievements  "),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
                MenuOption::Achievements,
            ));

            // 玩法介绍
            parent.spawn((
                Text::new("  How to Play  "),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
                MenuOption::HowToPlay,
            ));

            // 操作说明
            parent.spawn((
                Text::new("Up/Down: Select  |  Enter: Confirm  |  Left/Right: Change Mode"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node::default(),
            ));
        });
}

/// 游戏设置
fn setup_game(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut difficulty: ResMut<Difficulty>,
    mut game_timer: ResMut<GameTimer>,
    mut combo: ResMut<Combo>,
    settings: Res<crate::settings::GameSettings>,
    mut player_life: ResMut<crate::life::PlayerLife>,
    mut life_config: ResMut<crate::life::LifeConfig>,
    current_mode: Res<crate::game_mode::CurrentGameMode>,
    time_attack_timer: Res<crate::game_mode::TimeAttackTimer>,
) {
    // 重置游戏状态
    score.value = 0;
    difficulty.level = 1;
    // 应用难度设置
    difficulty.speed_multiplier = settings.difficulty.speed_multiplier();
    difficulty.spawn_interval_multiplier = settings.difficulty.spawn_interval_multiplier();
    game_timer.elapsed = 0.0;
    game_timer.last_difficulty_increase = 0.0;

    // 重置连击
    combo.count = 0;
    combo.combo_multiplier = 1.0;
    combo.timer = 0.0;

    // 根据游戏模式设置生命值
    use crate::game_mode::GameMode;
    match current_mode.mode {
        GameMode::Classic => {
            life_config.max_lives = settings.difficulty.lives();
            player_life.lives = life_config.max_lives;
        }
        GameMode::Endless => {
            life_config.max_lives = 1;
            player_life.lives = 1;
        }
        GameMode::TimeAttack => {
            life_config.max_lives = settings.difficulty.lives();
            player_life.lives = life_config.max_lives;
        }
    }

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

            // 连击显示
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.8, 0.0)),
                Node {
                    margin: UiRect::px(10.0, 10.0, 0.0, 0.0),
                    ..default()
                },
                ComboText,
            ));

            // 限时模式计时器显示
            if current_mode.mode == GameMode::TimeAttack {
                parent.spawn((
                    Text::new(format!("Time: {:.1}s", time_attack_timer.remaining)),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.5, 0.0)),
                    Node {
                        margin: UiRect::px(10.0, 10.0, 0.0, 0.0),
                        ..default()
                    },
                    TimerText,
                ));
            }

            // 生命显示
            crate::life::spawn_life_ui(parent);
        });
}

/// 清理游戏
fn cleanup_game(mut commands: Commands, query: Query<Entity, With<GameEntity>>) {
    // 只删除父实体，子实体会自动被递归删除
    // 使用 try_despawn 避免与 move_obstacles/check_collisions 同帧竞争时警告
    for entity in query.iter() {
        commands.entity(entity).try_despawn();
    }
}

/// 清理菜单
fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<MenuSelection>();
}

/// 游戏结束界面
fn setup_game_over(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut achievement_tracker: ResMut<crate::achievement::AchievementTracker>,
    mut save_data: ResMut<crate::save::SaveData>,
) {
    // 从存档中获取最高分
    score.high_score = save_data.high_score;
    if score.value > score.high_score {
        score.high_score = score.value;
    }

    // 记录游戏结束成就
    crate::achievement::record_game_over(&mut achievement_tracker, &mut save_data);

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
    mut selection: ResMut<MenuSelection>,
    mut query: Query<(&mut Text, &mut TextColor, &MenuOption)>,
    mut current_mode: ResMut<crate::game_mode::CurrentGameMode>,
) {
    // 上下选择
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        selection.0 = handle_menu_navigation(selection.0, -1, MENU_OPTION_COUNT);
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        selection.0 = handle_menu_navigation(selection.0, 1, MENU_OPTION_COUNT);
    }

    // 左右切换游戏模式
    if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::ArrowRight) {
        use crate::game_mode::GameMode;
        current_mode.mode = match current_mode.mode {
            GameMode::Classic => if keyboard.just_pressed(KeyCode::ArrowRight) { GameMode::Endless } else { GameMode::TimeAttack },
            GameMode::Endless => if keyboard.just_pressed(KeyCode::ArrowRight) { GameMode::TimeAttack } else { GameMode::Classic },
            GameMode::TimeAttack => if keyboard.just_pressed(KeyCode::ArrowRight) { GameMode::Classic } else { GameMode::Endless },
        };
    }

    // 确认选择
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        match selection.0 {
            0 => next_state.set(GameState::Playing),
            1 => {}, // GameMode - handled by left/right
            2 => next_state.set(GameState::Settings),
            3 => next_state.set(GameState::Achievements),
            4 => next_state.set(GameState::Help),
            _ => {}
        }
    }

    // 更新UI显示和高亮
    for (mut text, mut color, option) in query.iter_mut() {
        let idx = match option {
            MenuOption::Start => 0,
            MenuOption::GameMode => 1,
            MenuOption::Settings => 2,
            MenuOption::Achievements => 3,
            MenuOption::HowToPlay => 4,
        };

        let is_selected = idx == selection.0;

        // 更新文本（添加或移除选择指示符）
        let base_text = match option {
            MenuOption::Start => "Start Game".to_string(),
            MenuOption::GameMode => format!("Mode: {}", crate::game_mode::get_mode_name(current_mode.mode)),
            MenuOption::Settings => "Settings".to_string(),
            MenuOption::Achievements => "Achievements".to_string(),
            MenuOption::HowToPlay => "How to Play".to_string(),
        };
        **text = if is_selected {
            format!("> {} <", base_text)
        } else {
            format!("  {}  ", base_text)
        };

        // 更新颜色
        *color = if is_selected {
            TextColor(Color::srgb(1.0, 1.0, 1.0))
        } else {
            TextColor(Color::srgb(0.7, 0.7, 0.7))
        };
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
    combo: Res<Combo>,
    active_powerups: Res<crate::powerup::ActivePowerUps>,
    time: Res<Time>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    game_timer.elapsed += time.delta_secs();

    let base_score = calculate_base_score(game_timer.elapsed);
    if base_score != score.value {
        score.value += calculate_score_increment(
            base_score,
            score.value,
            combo.combo_multiplier,
            active_powerups.has_double_score,
        );

        // 更新UI
        if let Ok(mut text) = query.single_mut() {
            **text = format!("Score: {}", score.value);
        }
    }
}

/// 更新连击
fn update_combo(
    mut combo: ResMut<Combo>,
    mut query: Query<&mut Text, With<ComboText>>,
    time: Res<Time>,
) {
    if combo.count > 0 {
        combo.timer -= time.delta_secs();
        if combo.timer <= 0.0 {
            // 连击超时，重置
            combo.count = 0;
            combo.combo_multiplier = 1.0;
            combo.timer = 0.0;

            // 清空连击显示
            if let Ok(mut text) = query.single_mut() {
                **text = String::new();
            }
        } else {
            // 更新连击显示
            if let Ok(mut text) = query.single_mut() {
                **text = format!("Combo x{} ({:.1}s)", combo.count, combo.timer);
            }
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

/// 暂停系统
fn pause_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::KeyP) {
        next_state.set(GameState::Paused);
    }
}

/// 恢复系统
fn resume_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::KeyP) {
        next_state.set(GameState::Playing);
    }
    if keyboard.just_pressed(KeyCode::KeyQ) {
        next_state.set(GameState::Menu);
    }
}

/// 设置暂停菜单
fn setup_pause_menu(mut commands: Commands) {
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            PauseUI,
        ))
        .with_children(|parent| {
            // PAUSED 标题
            parent.spawn((
                Text::new("PAUSED"),
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

            // Resume 提示
            parent.spawn((
                Text::new("Press ESC or P to Resume"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(15.0)),
                    ..default()
                },
            ));

            // Quit 提示
            parent.spawn((
                Text::new("Press Q to Quit to Menu"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node::default(),
            ));
        });
}

/// 清理暂停菜单
fn cleanup_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 游戏实体标记
#[derive(Component)]
pub struct GameEntity;

/// 游戏UI标记
#[derive(Component)]
struct GameUI;

/// 菜单UI标记
#[derive(Component)]
struct MenuUI;

/// 游戏结束UI标记
#[derive(Component)]
struct GameOverUI;

/// 暂停UI标记
#[derive(Component)]
struct PauseUI;

/// 帮助UI标记
#[derive(Component)]
struct HelpUI;

/// 分数文本标记
#[derive(Component)]
pub struct ScoreText;

/// 连击文本标记
#[derive(Component)]
pub struct ComboText;

/// 计时器文本标记
#[derive(Component)]
pub struct TimerText;

/// 玩法介绍界面
fn setup_help(mut commands: Commands) {
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
            HelpUI,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("HOW TO PLAY"),
                TextFont { font_size: 36.0, ..default() },
                TextColor(Color::srgb(1.0, 1.0, 0.0)),
                Node { margin: UiRect::bottom(Val::Px(20.0)), ..default() },
            ));

            parent.spawn((
                Text::new("Arrow Keys: Move left/right to dodge obstacles"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node { margin: UiRect::bottom(Val::Px(6.0)), ..default() },
            ));

            parent.spawn((
                Text::new("Dodge obstacles to build combos and earn higher scores"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node { margin: UiRect::bottom(Val::Px(16.0)), ..default() },
            ));

            parent.spawn((
                Text::new("POWER-UPS"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::srgb(1.0, 1.0, 0.0)),
                Node { margin: UiRect::bottom(Val::Px(12.0)), ..default() },
            ));

            let powerups: &[(&str, &str)] = &[
                ("Shield", "Blocks one collision (5s, cyan)"),
                ("Clear", "Removes all obstacles (instant, orange)"),
                ("Magnet", "Attracts nearby items (8s, purple)"),
                ("Slowdown", "Halves obstacle speed (6s, blue)"),
                ("DoubleScore", "Doubles score gain (10s, gold)"),
                ("Shrink", "Halves your hitbox (5s, green)"),
                ("NitroBoost", "Faster lateral movement (3s, orange-red)"),
            ];

            for (name, desc) in powerups {
                parent.spawn((
                    Text::new(format!("{} - {}", name, desc)),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
                ));
            }

            parent.spawn((
                Text::new("Press ESC to go back"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node { margin: UiRect::top(Val::Px(20.0)), ..default() },
            ));
        });
}

/// 清理帮助界面
fn cleanup_help(mut commands: Commands, query: Query<Entity, With<HelpUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 帮助界面系统
fn help_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Menu);
    }
}
