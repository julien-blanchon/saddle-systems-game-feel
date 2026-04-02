use bevy::{post_process::effect_stack::ChromaticAberration, prelude::*};
use saddle_systems_game_feel::{
    EntityTimeScale, GameFeelDiagnostics, GlobalTimeScale, IgnoreGlobalTimeScale, IgnoreHitstop,
    PunchListener, ScreenPulseListener, ShakeListener, resolve_effective_time_scale,
};

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

pub fn add_example_plugins(app: &mut App, title: &str, clear_color: Color) {
    app.insert_resource(ClearColor(clear_color));
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: title.into(),
            resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
            ..default()
        }),
        ..default()
    }));
}

pub fn setup_2d_scene(mut commands: Commands) {
    commands.spawn((
        Name::new("Feel Camera"),
        DemoCamera,
        Camera2d,
        ShakeListener::default(),
        PunchListener::default(),
        ScreenPulseListener::default(),
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
        "{}\nGlobal scale {:.2} (base {:.2}, hitstop {:.2})\nActive shake {}  punch {}  flashes {}  screen {}\nScale fx {}  recipes {}\nBlue orb ignores global scaling for reference",
        label.0,
        global.scale,
        global.base_scale,
        global.hitstop_scale,
        diagnostics.active_shake_listeners,
        diagnostics.active_punch_listeners,
        diagnostics.active_entity_flashes,
        diagnostics.active_screen_pulses,
        diagnostics.active_scale_effects,
        diagnostics.active_recipe_players,
    );
}
