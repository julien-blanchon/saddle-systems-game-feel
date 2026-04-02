use bevy::{post_process::effect_stack::ChromaticAberration, prelude::*};
use saddle_systems_game_feel::{
    AddTrauma, GameFeelPlugin, ImpulseSpace, ListenerTarget, PunchListener, RequestCameraImpulse,
    ScreenPulseListener, ShakeListener,
};

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

#[derive(Resource)]
struct RecoilTimer(Timer);

#[derive(Component)]
struct RecoilCamera;

#[derive(Component)]
struct MovingCrate;

#[derive(Component)]
struct ExampleHud;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.03, 0.04, 0.07)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Game Feel 3D Recoil".into(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GameFeelPlugin::default())
        .insert_resource(RecoilTimer(Timer::from_seconds(0.70, TimerMode::Repeating)))
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_target, trigger_recoil, update_hud))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Recoil Camera"),
        RecoilCamera,
        Camera3d::default(),
        ShakeListener::default(),
        PunchListener::default(),
        ScreenPulseListener::default(),
        ChromaticAberration::default(),
        Transform::from_xyz(0.0, 1.8, 6.0).looking_at(Vec3::new(0.0, 1.2, 0.0), Vec3::Y),
    ));

    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 18_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.7, -0.6, 0.0)),
    ));

    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Cuboid::new(14.0, 0.2, 14.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 0.18, 0.22),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));

    commands.spawn((
        Name::new("Impact Crate"),
        MovingCrate,
        Mesh3d(meshes.add(Cuboid::new(1.4, 1.4, 1.4))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.92, 0.48, 0.28),
            perceptual_roughness: 0.45,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.9, 0.0),
    ));

    commands.spawn((
        Name::new("Backdrop Left"),
        Mesh3d(meshes.add(Cuboid::new(1.2, 2.8, 1.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.29, 0.72, 0.92),
            perceptual_roughness: 0.55,
            ..default()
        })),
        Transform::from_xyz(-2.6, 1.2, -1.6),
    ));

    commands.spawn((
        Name::new("Backdrop Right"),
        Mesh3d(meshes.add(Cuboid::new(1.0, 2.2, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.94, 0.82, 0.40),
            perceptual_roughness: 0.50,
            ..default()
        })),
        Transform::from_xyz(2.9, 1.0, -2.1),
    ));

    commands.spawn((
        Name::new("HUD"),
        ExampleHud,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(18.0),
            width: Val::Px(430.0),
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.04, 0.08, 0.78)),
        Text::new(
            "3D Recoil\nSelf-running local-space punch, micro trauma, and FOV pulse on a perspective camera.",
        ),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

fn animate_target(time: Res<Time>, mut query: Query<&mut Transform, With<MovingCrate>>) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let t = time.elapsed_secs();
    transform.translation.x = (t * 0.9).sin() * 1.3;
    transform.translation.y = 0.9 + (t * 1.7).sin() * 0.18;
    transform.rotate_y(0.8 * time.delta_secs());
}

fn trigger_recoil(
    time: Res<Time>,
    mut timer: ResMut<RecoilTimer>,
    camera: Query<Entity, With<RecoilCamera>>,
    mut trauma: MessageWriter<AddTrauma>,
    mut impulse: MessageWriter<RequestCameraImpulse>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let Ok(camera) = camera.single() else {
        return;
    };

    trauma.write(AddTrauma {
        target: ListenerTarget::Entity(camera),
        trauma: 0.10,
        origin: None,
        attenuation: None,
        propagation_speed: None,
        directional_bias: Vec3::new(0.02, -0.01, 0.0),
        profile_override: None,
    });
    impulse.write(RequestCameraImpulse {
        target: ListenerTarget::Entity(camera),
        translation: Vec3::new(0.0, 0.02, 0.12),
        rotation: Vec3::new(-0.10, 0.02, 0.0),
        fov: 0.03,
        origin: None,
        attenuation: None,
        propagation_speed: None,
        space: ImpulseSpace::Local,
        profile_override: None,
    });
}

fn update_hud(
    diagnostics: Res<saddle_systems_game_feel::GameFeelDiagnostics>,
    global: Res<saddle_systems_game_feel::GlobalTimeScale>,
    mut text: Query<&mut Text, With<ExampleHud>>,
) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };

    text.0 = format!(
        "3D Recoil\nGlobal scale {:.2}\nActive punch {}  shake {}  screen {}\nPerspective FOV punch is applied additively and restored every frame.",
        global.scale,
        diagnostics.active_punch_listeners,
        diagnostics.active_shake_listeners,
        diagnostics.active_screen_pulses,
    );
}
