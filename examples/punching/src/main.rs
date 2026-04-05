use saddle_systems_game_feel_example_support as support;

use bevy::prelude::*;
use saddle_systems_game_feel::{
    AddTrauma, GameFeelChannels, GameFeelPlugin, KnockbackReceiver, KnockbackState, ListenerTarget,
    PlayFeedbackRecipe, RequestCameraImpulse, RequestFlash, RequestHitstop, RequestKnockback,
    RequestSquashStretch,
};

#[derive(Component)]
struct PunchBag {
    base_translation: Vec3,
}

fn main() {
    let mut app = App::new();
    support::add_example_plugins(
        &mut app,
        "Game Feel: Punching",
        Color::srgb(0.05, 0.07, 0.11),
    );
    support::seed_example_pane(
        &mut app,
        support::ExampleFeelPane {
            interval_secs: 2.0,
            ..default()
        },
    );
    app.add_plugins(GameFeelPlugin::default());
    app.insert_resource(support::HudLabel(
        "Punching\nPress [SPACE] to punch the bag. Full game feel stack:\nscreenshake + hitstop + flash + squash + knockback + zoom".into(),
    ));
    app.add_systems(Startup, setup_scene);
    app.add_systems(
        Update,
        (
            support::advance_demo_motion,
            support::update_hud,
            handle_punch_input,
            apply_knockback_displacement,
        ),
    );
    app.run();
}

fn setup_scene(mut commands: Commands) {
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

    let bag_pos = Vec3::new(0.0, -30.0, 0.0);
    commands.spawn((
        Name::new("Punch Bag"),
        PunchBag {
            base_translation: bag_pos,
        },
        support::DemoTarget,
        Sprite::from_color(Color::srgb(0.92, 0.38, 0.28), Vec2::new(160.0, 200.0)),
        Transform::from_translation(bag_pos),
        KnockbackReceiver {
            max_displacement: 200.0,
        },
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

fn handle_punch_input(
    keys: Res<ButtonInput<KeyCode>>,
    pane: Res<support::ExampleFeelPane>,
    camera: Query<Entity, With<support::DemoCamera>>,
    target: Query<Entity, With<PunchBag>>,
    mut trauma: MessageWriter<AddTrauma>,
    mut impulse: MessageWriter<RequestCameraImpulse>,
    mut flash: MessageWriter<RequestFlash>,
    mut hitstop: MessageWriter<RequestHitstop>,
    mut squash: MessageWriter<RequestSquashStretch>,
    mut knockback: MessageWriter<RequestKnockback>,
    mut recipe: MessageWriter<PlayFeedbackRecipe>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }

    let Ok(camera) = camera.single() else {
        return;
    };
    let Ok(target) = target.single() else {
        return;
    };

    let intensity = pane.impulse_scale;

    trauma.write(
        AddTrauma::new(ListenerTarget::Entity(camera), 0.30 * intensity)
            .with_directional_bias(Vec3::new(0.15, -0.05, 0.0) * intensity),
    );

    impulse.write(
        RequestCameraImpulse::new(
            ListenerTarget::Entity(camera),
            Vec3::new(-0.18, 0.06, 0.0) * intensity,
        )
        .with_rotation(Vec3::new(0.0, 0.0, -0.10) * intensity)
        .with_fov(0.04 * intensity),
    );

    flash.write(
        RequestFlash::new(
            saddle_systems_game_feel::FlashTarget::EntityAndScreen {
                entity: target,
                screen: ListenerTarget::Entity(camera),
            },
            Color::WHITE,
            0.55 * pane.flash_scale,
        )
        .with_chromatic_aberration(0.08 * pane.chromatic_scale)
        .with_vignette(0.18 * pane.vignette_scale),
    );

    hitstop.write(
        RequestHitstop::new(
            saddle_systems_game_feel::TimeScaleTarget::World,
            pane.hitstop_hold_frames.round() as u32,
        )
        .with_recovery(pane.hitstop_recovery_frames.round() as u32),
    );

    squash.write(RequestSquashStretch::new(target, Vec3::new(1.25, 0.75, 1.0)).with_duration(0.20));

    knockback.write(RequestKnockback::new(target, Vec3::X, 120.0 * intensity).with_duration(0.35));

    recipe.write(
        PlayFeedbackRecipe::new("heavy_impact")
            .with_listener(camera)
            .with_target(target)
            .with_channels(GameFeelChannels::GAMEPLAY)
            .with_intensity(intensity * 0.5),
    );
}

fn apply_knockback_displacement(mut query: Query<(&PunchBag, &KnockbackState, &mut Transform)>) {
    for (bag, knockback, mut transform) in &mut query {
        transform.translation = bag.base_translation + knockback.displacement;
    }
}
