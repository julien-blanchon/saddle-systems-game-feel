use saddle_systems_game_feel_example_support as support;

use bevy::prelude::*;
use saddle_systems_game_feel::{
    AddTrauma, GameFeelPlugin, GameFeelToggles, ListenerTarget, RequestCameraImpulse, RequestFlash,
    RequestHitstop, RequestSquashStretch,
};

#[derive(Resource)]
struct ComparisonTimer(Timer);

#[derive(Component)]
struct LeftTarget;

#[derive(Component)]
struct RightTarget;

fn main() {
    let mut app = App::new();
    support::add_example_plugins(
        &mut app,
        "Game Feel: Comparison",
        Color::srgb(0.05, 0.07, 0.11),
    );
    support::seed_example_pane(
        &mut app,
        support::ExampleFeelPane {
            interval_secs: 1.0,
            ..default()
        },
    );
    app.add_plugins(GameFeelPlugin::default());
    app.insert_resource(ComparisonTimer(Timer::from_seconds(
        1.0,
        TimerMode::Repeating,
    )));
    app.insert_resource(support::HudLabel(
        "Comparison\nLeft: No game feel. Right: Full game feel.\nSame action, same timing. Press [T] to toggle effects.".into(),
    ));
    app.add_systems(Startup, setup_comparison_scene);
    app.add_systems(
        Update,
        (support::update_hud, toggle_effects, comparison_pulse),
    );
    app.run();
}

fn setup_comparison_scene(mut commands: Commands) {
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
        Sprite::from_color(Color::srgb(0.08, 0.10, 0.14), Vec2::new(1_600.0, 900.0)),
        Transform::from_xyz(0.0, 0.0, -20.0),
    ));

    commands.spawn((
        Name::new("Divider"),
        Sprite::from_color(Color::srgb(0.20, 0.22, 0.28), Vec2::new(4.0, 900.0)),
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));

    commands.spawn((
        Name::new("Left Label"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(160.0),
            bottom: Val::Px(60.0),
            ..default()
        },
        Text::new("NO GAME FEEL"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.4)),
    ));

    commands.spawn((
        Name::new("Right Label"),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(160.0),
            bottom: Val::Px(60.0),
            ..default()
        },
        Text::new("FULL GAME FEEL"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::srgba(0.4, 1.0, 0.6, 0.7)),
    ));

    commands.spawn((
        Name::new("Left Target"),
        LeftTarget,
        Sprite::from_color(Color::srgb(0.80, 0.35, 0.30), Vec2::new(120.0, 120.0)),
        Transform::from_xyz(-280.0, -30.0, 0.0),
    ));

    commands.spawn((
        Name::new("Right Target"),
        RightTarget,
        support::DemoTarget,
        Sprite::from_color(Color::srgb(0.80, 0.35, 0.30), Vec2::new(120.0, 120.0)),
        Transform::from_xyz(280.0, -30.0, 0.0),
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

fn toggle_effects(keys: Res<ButtonInput<KeyCode>>, mut toggles: ResMut<GameFeelToggles>) {
    if keys.just_pressed(KeyCode::KeyT) {
        let any_enabled = toggles.shake_enabled;
        if any_enabled {
            *toggles = GameFeelToggles::all_disabled();
        } else {
            *toggles = GameFeelToggles::all_enabled();
        }
    }
}

fn comparison_pulse(
    time: Res<Time>,
    pane: Res<support::ExampleFeelPane>,
    mut timer: ResMut<ComparisonTimer>,
    camera: Query<Entity, With<support::DemoCamera>>,
    left: Query<&mut Transform, (With<LeftTarget>, Without<RightTarget>)>,
    right: Query<Entity, With<RightTarget>>,
    mut trauma: MessageWriter<AddTrauma>,
    mut impulse: MessageWriter<RequestCameraImpulse>,
    mut flash: MessageWriter<RequestFlash>,
    mut hitstop: MessageWriter<RequestHitstop>,
    mut squash: MessageWriter<RequestSquashStretch>,
) {
    timer.0.set_duration(std::time::Duration::from_secs_f32(
        pane.interval_secs.max(0.3),
    ));
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    let Ok(camera) = camera.single() else {
        return;
    };
    let Ok(right_entity) = right.single() else {
        return;
    };

    let _ = left;

    trauma.write(
        AddTrauma::new(ListenerTarget::Entity(camera), 0.25 * pane.trauma_scale)
            .with_directional_bias(Vec3::new(0.10, -0.04, 0.0)),
    );

    impulse.write(
        RequestCameraImpulse::new(
            ListenerTarget::Entity(camera),
            Vec3::new(-0.12, 0.04, 0.0) * pane.impulse_scale,
        )
        .with_rotation(Vec3::new(0.0, 0.0, -0.08))
        .with_fov(0.03),
    );

    flash.write(
        RequestFlash::new(
            saddle_systems_game_feel::FlashTarget::EntityAndScreen {
                entity: right_entity,
                screen: ListenerTarget::Entity(camera),
            },
            Color::WHITE,
            0.45 * pane.flash_scale,
        )
        .with_chromatic_aberration(0.06 * pane.chromatic_scale)
        .with_vignette(0.14 * pane.vignette_scale),
    );

    hitstop.write(
        RequestHitstop::new(
            saddle_systems_game_feel::TimeScaleTarget::World,
            pane.hitstop_hold_frames.round() as u32,
        )
        .with_recovery(pane.hitstop_recovery_frames.round() as u32),
    );

    squash.write(
        RequestSquashStretch::new(right_entity, Vec3::new(1.20, 0.80, 1.0)).with_duration(0.18),
    );
}
