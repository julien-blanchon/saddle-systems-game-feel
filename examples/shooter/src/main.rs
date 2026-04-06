use saddle_systems_game_feel_example_support as support;

use bevy::prelude::*;
use saddle_systems_game_feel::{GameFeelPlugin, PlayFeedbackRecipe, presets};

#[derive(Component)]
struct Crosshair;

#[derive(Component)]
struct ShooterTarget;

#[derive(Resource)]
struct FireCooldown(Timer);

fn main() {
    let mut app = App::new();
    support::add_example_plugins(
        &mut app,
        "Game Feel: Shooter",
        Color::srgb(0.04, 0.06, 0.10),
    );
    support::seed_example_pane(
        &mut app,
        support::ExampleFeelPane {
            interval_secs: 2.0,
            ..default()
        },
    );
    app.add_plugins(GameFeelPlugin::default());
    app.insert_resource(presets::recipes::library());
    app.insert_resource(FireCooldown(Timer::from_seconds(0.12, TimerMode::Once)));
    app.insert_resource(support::HudLabel(
        "Shooter Feel\n[SPACE] Fire weapon  [1] Light hit  [2] Heavy impact  [3] Parry\nRecoil, muzzle flash, hitstop on target".into(),
    ));
    app.add_systems(Startup, setup_shooter_scene);
    app.add_systems(
        Update,
        (
            support::advance_demo_motion,
            support::update_hud,
            fire_weapon,
        ),
    );
    app.run();
}

fn setup_shooter_scene(mut commands: Commands) {
    commands.spawn((
        Name::new("Feel Camera"),
        support::DemoCamera,
        Camera2d,
        saddle_systems_game_feel::ShakeListener::default(),
        saddle_systems_game_feel::PunchListener::default(),
        saddle_systems_game_feel::ScreenPulseListener::default(),
        saddle_systems_game_feel::RumbleListener::default(),
        bevy::post_process::effect_stack::ChromaticAberration::default(),
        Transform::from_xyz(0.0, 0.0, 10.0),
    ));

    commands.spawn((
        Name::new("Backdrop"),
        Sprite::from_color(Color::srgb(0.06, 0.08, 0.12), Vec2::new(1_600.0, 900.0)),
        Transform::from_xyz(0.0, 0.0, -20.0),
    ));

    for (index, (x, y, color)) in [
        (180.0_f32, 40.0_f32, Color::srgb(0.90, 0.35, 0.30)),
        (-160.0, -80.0, Color::srgb(0.30, 0.75, 0.90)),
        (320.0, -120.0, Color::srgb(0.90, 0.80, 0.30)),
    ]
    .into_iter()
    .enumerate()
    {
        let pos = Vec3::new(x, y, 0.0);
        commands.spawn((
            Name::new(format!("Target {}", index + 1)),
            ShooterTarget,
            support::DemoTarget,
            Sprite::from_color(color, Vec2::new(100.0, 100.0)),
            Transform::from_translation(pos),
            support::DemoMotion {
                base_translation: pos,
                amplitude: Vec3::new(40.0, 25.0, 0.0),
                frequency_hz: 0.25 + index as f32 * 0.10,
                phase: index as f32 * 1.5,
            },
            support::MotionClock::default(),
        ));
    }

    commands.spawn((
        Name::new("Crosshair H"),
        Crosshair,
        Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.5), Vec2::new(24.0, 2.0)),
        Transform::from_xyz(0.0, -200.0, 5.0),
    ));
    commands.spawn((
        Name::new("Crosshair V"),
        Crosshair,
        Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.5), Vec2::new(2.0, 24.0)),
        Transform::from_xyz(0.0, -200.0, 5.0),
    ));

    commands.spawn((
        Name::new("Demo HUD"),
        support::DemoHud,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(18.0),
            width: Val::Px(520.0),
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

fn fire_weapon(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    pane: Res<support::ExampleFeelPane>,
    camera: Query<Entity, With<support::DemoCamera>>,
    targets: Query<Entity, With<ShooterTarget>>,
    mut cooldown: ResMut<FireCooldown>,
    mut recipe: MessageWriter<PlayFeedbackRecipe>,
) {
    let tick = cooldown.0.tick(time.delta());

    let Ok(camera) = camera.single() else {
        return;
    };

    let first_target = targets.iter().next();
    let can_fire = tick.just_finished() || cooldown.0.fraction() >= 1.0;

    if keys.just_pressed(KeyCode::Space) && can_fire {
        cooldown.0.reset();

        recipe.write(
            PlayFeedbackRecipe::new(presets::recipes::WEAPON_FIRE)
                .with_listener(camera)
                .with_channels(presets::channels::WEAPON)
                .with_intensity(pane.impulse_scale),
        );

        if let Some(target) = first_target {
            recipe.write(
                PlayFeedbackRecipe::new(presets::recipes::LIGHT_HIT)
                    .with_listener(camera)
                    .with_target(target)
                    .with_channels(presets::channels::WEAPON)
                    .with_intensity(pane.impulse_scale),
            );
        }
    }

    if keys.just_pressed(KeyCode::Digit1)
        && let Some(target) = first_target
    {
        recipe.write(
            PlayFeedbackRecipe::new(presets::recipes::LIGHT_HIT)
                .with_listener(camera)
                .with_target(target)
                .with_channels(presets::channels::GAMEPLAY)
                .with_intensity(pane.impulse_scale),
        );
    }

    if keys.just_pressed(KeyCode::Digit2)
        && let Some(target) = first_target
    {
        recipe.write(
            PlayFeedbackRecipe::new(presets::recipes::HEAVY_IMPACT)
                .with_listener(camera)
                .with_target(target)
                .with_channels(presets::channels::GAMEPLAY)
                .with_intensity(pane.impulse_scale),
        );
    }

    if keys.just_pressed(KeyCode::Digit3)
        && let Some(target) = first_target
    {
        recipe.write(
            PlayFeedbackRecipe::new(presets::recipes::PARRY)
                .with_listener(camera)
                .with_target(target)
                .with_channels(presets::channels::GAMEPLAY)
                .with_intensity(pane.impulse_scale),
        );
    }
}
