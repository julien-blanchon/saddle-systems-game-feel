use bevy::{post_process::effect_stack::ChromaticAberration, prelude::*};
use bevy_flair::prelude::InlineStyle;
use saddle_systems_game_feel::{
    EntityTimeScale, GameFeelConfig, GameFeelDiagnostics, GlobalTimeScale,
    IgnoreGlobalTimeScale, IgnoreHitstop, PunchListener, RumbleListener, ScreenPulseListener,
    ScreenPulsePresentation, ShakeListener, resolve_effective_time_scale,
};
use saddle_pane::prelude::*;

const PANE_DARK_THEME_VARS: &[(&str, &str)] = &[
    ("--pane-elevation-1", "#28292e"),
    ("--pane-elevation-2", "#222327"),
    ("--pane-elevation-3", "rgba(187, 188, 196, 0.10)"),
    ("--pane-border", "#3c3d44"),
    ("--pane-border-focus", "#7090b0"),
    ("--pane-border-subtle", "#333438"),
    ("--pane-text-primary", "#bbbcc4"),
    ("--pane-text-secondary", "#78797f"),
    ("--pane-text-muted", "#5c5d64"),
    ("--pane-text-on-accent", "#ffffff"),
    ("--pane-text-brighter", "#d0d1d8"),
    ("--pane-text-monitor", "#9a9ba2"),
    ("--pane-text-log", "#8a8b92"),
    ("--pane-accent", "#4a6fa5"),
    ("--pane-accent-hover", "#5a8fd5"),
    ("--pane-accent-active", "#3a5f95"),
    ("--pane-accent-subtle", "rgba(74, 111, 165, 0.15)"),
    ("--pane-accent-fill", "rgba(74, 111, 165, 0.60)"),
    ("--pane-accent-fill-hover", "rgba(90, 143, 213, 0.70)"),
    ("--pane-accent-fill-active", "rgba(90, 143, 213, 0.80)"),
    ("--pane-accent-checked", "rgba(74, 111, 165, 0.25)"),
    ("--pane-accent-checked-hover", "rgba(74, 111, 165, 0.35)"),
    ("--pane-accent-indicator", "rgba(74, 111, 165, 0.80)"),
    ("--pane-accent-knob", "#7aacdf"),
    ("--pane-widget-bg", "rgba(187, 188, 196, 0.10)"),
    ("--pane-widget-hover", "rgba(187, 188, 196, 0.15)"),
    ("--pane-widget-focus", "rgba(187, 188, 196, 0.20)"),
    ("--pane-widget-active", "rgba(187, 188, 196, 0.25)"),
    ("--pane-widget-bg-muted", "rgba(187, 188, 196, 0.06)"),
    ("--pane-tab-hover-bg", "rgba(187, 188, 196, 0.06)"),
    ("--pane-hover-bg", "rgba(255, 255, 255, 0.03)"),
    ("--pane-active-bg", "rgba(255, 255, 255, 0.05)"),
    ("--pane-popup-bg", "#1e1f24"),
    ("--pane-bg-dark", "rgba(0, 0, 0, 0.25)"),
];

pub const WINDOW_WIDTH: u32 = 1280;
pub const WINDOW_HEIGHT: u32 = 720;

#[derive(Component)]
pub struct DemoCamera;

#[derive(Component)]
pub struct DemoTarget;

#[derive(Component)]
pub struct DemoCompanion;

#[derive(Component)]
pub struct DemoHud;

#[derive(Component)]
pub struct DemoMotion {
    pub base_translation: Vec3,
    pub amplitude: Vec3,
    pub frequency_hz: f32,
    pub phase: f32,
}

#[derive(Component, Default)]
pub struct MotionClock(pub f32);

#[derive(Resource)]
#[allow(dead_code)]
pub struct HudLabel(pub String);

#[derive(Resource, Debug, Clone, PartialEq, Pane)]
#[pane(title = "Game Feel", position = "top-right")]
pub struct ExampleFeelPane {
    #[pane(slider, min = 0.2, max = 2.5, step = 0.05)]
    pub interval_secs: f32,
    #[pane(slider, min = 0.2, max = 2.0, step = 0.05)]
    pub trauma_scale: f32,
    #[pane(slider, min = 0.2, max = 2.0, step = 0.05)]
    pub impulse_scale: f32,
    #[pane(slider, min = 0.2, max = 2.0, step = 0.05)]
    pub flash_scale: f32,
    #[pane(slider, min = 0.2, max = 2.0, step = 0.05)]
    pub chromatic_scale: f32,
    #[pane(slider, min = 0.2, max = 2.0, step = 0.05)]
    pub vignette_scale: f32,
    #[pane(slider, min = 0.0, max = 8.0, step = 1.0)]
    pub hitstop_hold_frames: f32,
    #[pane(slider, min = 0.0, max = 12.0, step = 1.0)]
    pub hitstop_recovery_frames: f32,
    #[pane(slider, min = 0.08, max = 0.4, step = 0.01)]
    pub overlay_border_fraction: f32,
    #[pane(slider, min = 0.0, max = 1.0, step = 1.0)]
    pub legacy_screen_fx: f32,
    #[pane(monitor)]
    pub active_screen_pulses: f32,
    #[pane(monitor)]
    pub active_rumble: f32,
    #[pane(monitor)]
    pub global_time_scale: f32,
}

impl Default for ExampleFeelPane {
    fn default() -> Self {
        Self {
            interval_secs: 1.0,
            trauma_scale: 1.0,
            impulse_scale: 1.0,
            flash_scale: 1.0,
            chromatic_scale: 1.0,
            vignette_scale: 1.0,
            hitstop_hold_frames: 3.0,
            hitstop_recovery_frames: 5.0,
            overlay_border_fraction: 0.18,
            legacy_screen_fx: 1.0,
            active_screen_pulses: 0.0,
            active_rumble: 0.0,
            global_time_scale: 1.0,
        }
    }
}

#[derive(Resource, Clone)]
struct ExampleFeelPaneBootstrap(ExampleFeelPane);

pub fn add_example_plugins(app: &mut App, title: &str, clear_color: Color) {
    app.insert_resource(ClearColor(clear_color));
    app.insert_resource(GameFeelConfig {
        screen_presentation: ScreenPulsePresentation::LegacyBuiltIn,
        ..default()
    });
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: title.into(),
            resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
            ..default()
        }),
        ..default()
    }));
    install_pane(app);
}

pub fn install_pane(app: &mut App) {
    if !app.is_plugin_added::<PanePlugin>() {
        app.add_plugins((
            bevy_flair::FlairPlugin,
            bevy_input_focus::InputDispatchPlugin,
            bevy_ui_widgets::UiWidgetsPlugins,
            bevy_input_focus::tab_navigation::TabNavigationPlugin,
            PanePlugin,
        ));
    }

    app.register_pane::<ExampleFeelPane>()
        .add_systems(
            PreUpdate,
            (prime_pane_theme_vars, apply_bootstrapped_pane, sync_game_feel_config).chain(),
        )
        .add_systems(PostUpdate, reflect_game_feel_monitors);
}

pub fn seed_example_pane(app: &mut App, pane: ExampleFeelPane) {
    app.insert_resource(ExampleFeelPaneBootstrap(pane));
}

pub fn setup_2d_scene(mut commands: Commands) {
    commands.spawn((
        Name::new("Feel Camera"),
        DemoCamera,
        Camera2d,
        ShakeListener::default(),
        PunchListener::default(),
        ScreenPulseListener::default(),
        RumbleListener::default(),
        ChromaticAberration::default(),
        Transform::from_xyz(0.0, 0.0, 10.0),
    ));

    commands.spawn((
        Name::new("Backdrop"),
        Sprite::from_color(Color::srgb(0.08, 0.10, 0.14), Vec2::new(1_600.0, 900.0)),
        Transform::from_xyz(0.0, 0.0, -20.0),
    ));

    commands.spawn((
        Name::new("Impact Dummy"),
        DemoTarget,
        Sprite::from_color(Color::srgb(0.92, 0.38, 0.28), Vec2::new(180.0, 180.0)),
        Transform::from_xyz(0.0, -30.0, 0.0),
        DemoMotion {
            base_translation: Vec3::new(0.0, -30.0, 0.0),
            amplitude: Vec3::new(48.0, 28.0, 0.0),
            frequency_hz: 0.35,
            phase: 0.0,
        },
        MotionClock::default(),
    ));

    commands.spawn((
        Name::new("Companion Orb"),
        DemoCompanion,
        IgnoreGlobalTimeScale,
        IgnoreHitstop,
        Sprite::from_color(Color::srgb(0.28, 0.76, 0.95), Vec2::new(72.0, 72.0)),
        Transform::from_xyz(260.0, 150.0, 0.0),
        DemoMotion {
            base_translation: Vec3::new(260.0, 150.0, 0.0),
            amplitude: Vec3::new(90.0, 42.0, 0.0),
            frequency_hz: 0.7,
            phase: 1.4,
        },
        MotionClock::default(),
    ));

    for (index, (translation, color, phase)) in [
        (
            Vec3::new(-360.0, 160.0, -1.0),
            Color::srgb(0.94, 0.79, 0.36),
            0.0,
        ),
        (
            Vec3::new(-300.0, -170.0, -1.0),
            Color::srgb(0.35, 0.88, 0.64),
            1.2,
        ),
        (
            Vec3::new(360.0, -140.0, -1.0),
            Color::srgb(0.96, 0.58, 0.78),
            2.4,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        commands.spawn((
            Name::new(format!("Accent Orb {}", index + 1)),
            Sprite::from_color(color, Vec2::new(46.0, 46.0)),
            Transform::from_translation(translation),
            DemoMotion {
                base_translation: translation,
                amplitude: Vec3::new(34.0, 20.0, 0.0),
                frequency_hz: 0.55 + index as f32 * 0.08,
                phase,
            },
            MotionClock::default(),
        ));
    }

    commands.spawn((
        Name::new("Demo HUD"),
        DemoHud,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(18.0),
            width: Val::Px(420.0),
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.04, 0.08, 0.76)),
        Text::default(),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

pub fn advance_demo_motion(
    global: Res<GlobalTimeScale>,
    mut query: Query<(
        &mut MotionClock,
        &DemoMotion,
        &mut Transform,
        Option<&EntityTimeScale>,
        Option<&IgnoreGlobalTimeScale>,
        Option<&IgnoreHitstop>,
    )>,
) {
    for (mut clock, motion, mut transform, local_scale, ignore_global, ignore_hitstop) in &mut query
    {
        let scale = resolve_effective_time_scale(
            &global,
            local_scale,
            ignore_global.is_some(),
            ignore_hitstop.is_some(),
        );
        clock.0 += global.unscaled_delta_secs * scale.max(0.0);
        let angle = clock.0 * motion.frequency_hz * std::f32::consts::TAU + motion.phase;
        transform.translation = motion.base_translation
            + Vec3::new(
                motion.amplitude.x * angle.cos(),
                motion.amplitude.y * (angle * 1.3).sin(),
                motion.amplitude.z * (angle * 0.7).sin(),
            );
    }
}

#[allow(dead_code)]
pub fn update_hud(
    label: Res<HudLabel>,
    global: Res<GlobalTimeScale>,
    diagnostics: Res<GameFeelDiagnostics>,
    mut query: Query<&mut Text, With<DemoHud>>,
) {
    let Ok(mut text) = query.single_mut() else {
        return;
    };

    text.0 = format!(
        "{}\nGlobal scale {:.2} (base {:.2}, hitstop {:.2})\nActive shake {}  punch {}  flashes {}  screen {}  rumble {}\nScale fx {}  recipes {}\nBlue orb ignores global scaling for reference",
        label.0,
        global.scale,
        global.base_scale,
        global.hitstop_scale,
        diagnostics.active_shake_listeners,
        diagnostics.active_punch_listeners,
        diagnostics.active_entity_flashes,
        diagnostics.active_screen_pulses,
        diagnostics.active_rumble_listeners,
        diagnostics.active_scale_effects,
        diagnostics.active_recipe_players,
    );
}

fn prime_pane_theme_vars(mut panes: Query<&mut InlineStyle, Added<PaneRoot>>) {
    for mut style in &mut panes {
        for &(key, value) in PANE_DARK_THEME_VARS {
            style.set(key, value.to_owned());
        }
    }
}

fn apply_bootstrapped_pane(
    bootstrap: Option<Res<ExampleFeelPaneBootstrap>>,
    mut pane: ResMut<ExampleFeelPane>,
) {
    let Some(bootstrap) = bootstrap else {
        return;
    };

    if *pane == ExampleFeelPane::default() {
        *pane = bootstrap.0.clone();
    }
}

fn sync_game_feel_config(pane: Res<ExampleFeelPane>, mut config: ResMut<GameFeelConfig>) {
    config.overlay_border_fraction = pane.overlay_border_fraction.clamp(0.02, 0.45);
    config.screen_presentation = if pane.legacy_screen_fx.round() as i32 == 0 {
        ScreenPulsePresentation::OutputOnly
    } else {
        ScreenPulsePresentation::LegacyBuiltIn
    };
}

fn reflect_game_feel_monitors(
    diagnostics: Res<GameFeelDiagnostics>,
    global: Res<GlobalTimeScale>,
    mut pane: ResMut<ExampleFeelPane>,
) {
    let pane = pane.bypass_change_detection();
    pane.active_screen_pulses = diagnostics.active_screen_pulses as f32;
    pane.active_rumble = diagnostics.active_rumble_listeners as f32;
    pane.global_time_scale = global.scale;
}
