use saddle_systems_game_feel_example_support as support;

use bevy::prelude::*;
use saddle_systems_game_feel::{
    AddTrauma, GameFeelChannels, GameFeelPlugin, ListenerTarget, PlayFeedbackRecipe,
    RequestCameraImpulse, RequestSquashStretch,
};

const GRAVITY: f32 = -1200.0;
const JUMP_VELOCITY: f32 = 500.0;
const MOVE_SPEED: f32 = 350.0;
const FLOOR_Y: f32 = -180.0;

#[derive(Component)]
struct Player {
    velocity: Vec2,
    grounded: bool,
}

#[derive(Component)]
struct Floor;

fn main() {
    let mut app = App::new();
    support::add_example_plugins(
        &mut app,
        "Game Feel: Platformer",
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
        "Platformer Feel\n[A/D] Move  [SPACE] Jump\nLanding squash, jump stretch, wall shake"
            .into(),
    ));
    app.add_systems(Startup, setup_platformer_scene);
    app.add_systems(
        Update,
        (support::update_hud, player_input, player_physics).chain(),
    );
    app.run();
}

fn setup_platformer_scene(mut commands: Commands) {
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
        Name::new("Floor"),
        Floor,
        Sprite::from_color(Color::srgb(0.18, 0.20, 0.24), Vec2::new(900.0, 40.0)),
        Transform::from_xyz(0.0, FLOOR_Y - 20.0, -5.0),
    ));

    commands.spawn((
        Name::new("Left Wall"),
        Sprite::from_color(Color::srgb(0.18, 0.20, 0.24), Vec2::new(40.0, 500.0)),
        Transform::from_xyz(-470.0, FLOOR_Y + 200.0, -5.0),
    ));

    commands.spawn((
        Name::new("Right Wall"),
        Sprite::from_color(Color::srgb(0.18, 0.20, 0.24), Vec2::new(40.0, 500.0)),
        Transform::from_xyz(470.0, FLOOR_Y + 200.0, -5.0),
    ));

    commands.spawn((
        Name::new("Player"),
        support::DemoTarget,
        Player {
            velocity: Vec2::ZERO,
            grounded: true,
        },
        Sprite::from_color(Color::srgb(0.40, 0.85, 0.55), Vec2::new(48.0, 64.0)),
        Transform::from_xyz(0.0, FLOOR_Y + 32.0, 0.0),
    ));

    commands.spawn((
        Name::new("Demo HUD"),
        support::DemoHud,
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

fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    pane: Res<support::ExampleFeelPane>,
    mut player: Query<(Entity, &mut Player, &Transform)>,
    camera: Query<Entity, With<support::DemoCamera>>,
    mut squash: MessageWriter<RequestSquashStretch>,
    mut trauma: MessageWriter<AddTrauma>,
    mut impulse: MessageWriter<RequestCameraImpulse>,
    mut recipe: MessageWriter<PlayFeedbackRecipe>,
) {
    let Ok((entity, mut player, transform)) = player.single_mut() else {
        return;
    };
    let Ok(camera) = camera.single() else {
        return;
    };

    let mut move_dir = 0.0;
    if keys.pressed(KeyCode::KeyA) {
        move_dir -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        move_dir += 1.0;
    }
    player.velocity.x = move_dir * MOVE_SPEED;

    if keys.just_pressed(KeyCode::Space) && player.grounded {
        player.velocity.y = JUMP_VELOCITY;
        player.grounded = false;

        squash.write(
            RequestSquashStretch::new(entity, Vec3::new(0.80, 1.30, 1.0)).with_duration(0.14),
        );

        trauma.write(AddTrauma::new(
            ListenerTarget::Entity(camera),
            0.06 * pane.trauma_scale,
        ));

        impulse.write(RequestCameraImpulse::new(
            ListenerTarget::Entity(camera),
            Vec3::new(0.0, 0.04, 0.0) * pane.impulse_scale,
        ));

        recipe.write(
            PlayFeedbackRecipe::new("dash_burst")
                .with_listener(camera)
                .with_target(entity)
                .with_channels(GameFeelChannels::GAMEPLAY)
                .with_intensity(0.3),
        );
    }

    let _ = transform;
}

fn player_physics(
    time: Res<Time>,
    pane: Res<support::ExampleFeelPane>,
    mut player: Query<(Entity, &mut Player, &mut Transform)>,
    camera: Query<Entity, With<support::DemoCamera>>,
    mut squash: MessageWriter<RequestSquashStretch>,
    mut trauma: MessageWriter<AddTrauma>,
    mut impulse: MessageWriter<RequestCameraImpulse>,
    mut recipe: MessageWriter<PlayFeedbackRecipe>,
) {
    let Ok((entity, mut player, mut transform)) = player.single_mut() else {
        return;
    };
    let Ok(camera) = camera.single() else {
        return;
    };

    let dt = time.delta_secs();

    if !player.grounded {
        player.velocity.y += GRAVITY * dt;
    }

    transform.translation.x += player.velocity.x * dt;
    transform.translation.y += player.velocity.y * dt;

    let wall_left = -450.0 + 24.0;
    let wall_right = 450.0 - 24.0;

    if transform.translation.x < wall_left {
        transform.translation.x = wall_left;
        player.velocity.x = 0.0;
        trauma.write(
            AddTrauma::new(ListenerTarget::Entity(camera), 0.12 * pane.trauma_scale)
                .with_directional_bias(Vec3::new(-0.08, 0.0, 0.0)),
        );
    }
    if transform.translation.x > wall_right {
        transform.translation.x = wall_right;
        player.velocity.x = 0.0;
        trauma.write(
            AddTrauma::new(ListenerTarget::Entity(camera), 0.12 * pane.trauma_scale)
                .with_directional_bias(Vec3::new(0.08, 0.0, 0.0)),
        );
    }

    if transform.translation.y <= FLOOR_Y + 32.0 && !player.grounded {
        transform.translation.y = FLOOR_Y + 32.0;
        let impact_velocity = player.velocity.y.abs();
        player.velocity.y = 0.0;
        player.grounded = true;

        let impact_intensity = (impact_velocity / JUMP_VELOCITY).clamp(0.0, 1.0);

        squash.write(
            RequestSquashStretch::new(entity, Vec3::new(1.25, 0.75, 1.0)).with_duration(0.20),
        );

        if impact_intensity > 0.3 {
            recipe.write(
                PlayFeedbackRecipe::new("landing_impact")
                    .with_listener(camera)
                    .with_target(entity)
                    .with_origin(transform.translation)
                    .with_channels(GameFeelChannels::GAMEPLAY)
                    .with_intensity(impact_intensity),
            );

            impulse.write(RequestCameraImpulse::new(
                ListenerTarget::Entity(camera),
                Vec3::new(0.0, -0.06, 0.0) * impact_intensity * pane.impulse_scale,
            ));
        }
    }

    if transform.translation.y > FLOOR_Y + 32.0 {
        player.grounded = false;
    }
}
