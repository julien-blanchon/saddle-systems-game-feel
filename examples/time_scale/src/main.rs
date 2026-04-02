use saddle_systems_game_feel_example_support as support;

use bevy::prelude::*;
use saddle_systems_game_feel::{GameFeelPlugin, RequestTimeScale, TimeScaleTarget};

#[derive(Resource)]
struct TimeScaleTimer(Timer);

fn main() {
    let mut app = App::new();
    support::add_example_plugins(
        &mut app,
        "Game Feel Time Scale",
        Color::srgb(0.04, 0.05, 0.09),
    );
    app.add_plugins(GameFeelPlugin::default());
    app.insert_resource(TimeScaleTimer(Timer::from_seconds(
        1.6,
        TimerMode::Repeating,
    )));
    app.insert_resource(support::HudLabel(
        "Time Scale\nGlobal slow-mo pulses every 1.6s while the blue orb ignores it.".into(),
    ));
    app.add_systems(Startup, support::setup_2d_scene);
    app.add_systems(
        Update,
        (
            support::advance_demo_motion,
            support::update_hud,
            pulse_time_scale,
        ),
    );
    app.run();
}

fn pulse_time_scale(
    time: Res<Time>,
    mut timer: ResMut<TimeScaleTimer>,
    target: Query<Entity, With<support::DemoTarget>>,
    mut requests: MessageWriter<RequestTimeScale>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    requests.write(RequestTimeScale {
        target: TimeScaleTarget::World,
        scale: 0.35,
        ramp_in_secs: 0.04,
        hold_secs: 0.14,
        ramp_out_secs: 0.26,
        easing: bevy::math::curve::easing::EaseFunction::SineInOut,
        priority: 0,
        group_entities: Vec::new(),
    });

    if let Ok(target) = target.single() {
        requests.write(RequestTimeScale {
            target: TimeScaleTarget::Entity(target),
            scale: 0.55,
            ramp_in_secs: 0.0,
            hold_secs: 0.24,
            ramp_out_secs: 0.20,
            easing: bevy::math::curve::easing::EaseFunction::SineInOut,
            priority: 1,
            group_entities: Vec::new(),
        });
    }
}
