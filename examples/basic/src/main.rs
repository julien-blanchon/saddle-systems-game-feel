use saddle_systems_game_feel_example_support as support;

use bevy::prelude::*;
use saddle_systems_game_feel::{AddTrauma, GameFeelPlugin, ListenerTarget, RequestCameraImpulse};

#[derive(Resource)]
struct PulseTimer(Timer);

fn main() {
    let mut app = App::new();
    support::add_example_plugins(&mut app, "Game Feel Basic", Color::srgb(0.05, 0.07, 0.11));
    support::seed_example_pane(
        &mut app,
        support::ExampleFeelPane {
            interval_secs: 0.85,
            ..default()
        },
    );
    app.add_plugins(GameFeelPlugin::default());
    app.insert_resource(PulseTimer(Timer::from_seconds(0.85, TimerMode::Repeating)));
    app.insert_resource(support::HudLabel(
        "Basic\nAutomatic trauma shake and camera punch every 0.85s.".into(),
    ));
    app.add_systems(Startup, support::setup_2d_scene);
    app.add_systems(
        Update,
        (
            support::advance_demo_motion,
            support::update_hud,
            pulse_basic,
        ),
    );
    app.run();
}

fn pulse_basic(
    time: Res<Time>,
    pane: Res<support::ExampleFeelPane>,
    mut timer: ResMut<PulseTimer>,
    camera: Query<Entity, With<support::DemoCamera>>,
    mut trauma: MessageWriter<AddTrauma>,
    mut impulse: MessageWriter<RequestCameraImpulse>,
) {
    timer.0.set_duration(std::time::Duration::from_secs_f32(
        pane.interval_secs.max(0.2),
    ));
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    let Ok(camera) = camera.single() else {
        return;
    };

    trauma.write(AddTrauma {
        target: ListenerTarget::Entity(camera),
        trauma: 0.22 * pane.trauma_scale,
        origin: None,
        attenuation: None,
        propagation_speed: None,
        directional_bias: Vec3::new(0.06, 0.0, 0.0) * pane.impulse_scale,
        profile_override: None,
    });
    impulse.write(RequestCameraImpulse {
        target: ListenerTarget::Entity(camera),
        translation: Vec3::new(-0.08, 0.03, 0.0) * pane.impulse_scale,
        ..RequestCameraImpulse::new(ListenerTarget::Entity(camera), Vec3::ZERO)
    });
}
