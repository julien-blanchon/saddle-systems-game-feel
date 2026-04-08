#[cfg(feature = "e2e")]
mod e2e;
use saddle_systems_game_feel_example_support as support;

use bevy::prelude::*;
use saddle_systems_game_feel::{
    AddTrauma, FlashTarget, GameFeelPlugin, ListenerTarget, RequestFlash, RequestHitstop,
    RequestSquashStretch, TimeScaleTarget,
};

#[derive(Resource)]
struct ImpactTimer(Timer);

fn main() {
    let mut app = App::new();
    support::add_example_plugins(&mut app, "Game Feel Hitstop", Color::srgb(0.06, 0.05, 0.08));
    support::seed_example_pane(
        &mut app,
        support::ExampleFeelPane {
            interval_secs: 1.2,
            ..default()
        },
    );
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::HitstopE2EPlugin);
    app.add_plugins(GameFeelPlugin::default());
    app.insert_resource(ImpactTimer(Timer::from_seconds(1.2, TimerMode::Repeating)));
    app.insert_resource(support::HudLabel(
        "Hitstop + Flash\nThe red dummy freezes briefly while the blue orb keeps moving.".into(),
    ));
    app.add_systems(Startup, support::setup_2d_scene);
    app.add_systems(
        Update,
        (
            support::advance_demo_motion,
            support::update_hud,
            trigger_impact,
        ),
    );
    app.run();
}

fn trigger_impact(
    time: Res<Time>,
    pane: Res<support::ExampleFeelPane>,
    mut timer: ResMut<ImpactTimer>,
    camera: Query<Entity, With<support::DemoCamera>>,
    target: Query<Entity, With<support::DemoTarget>>,
    mut trauma: MessageWriter<AddTrauma>,
    mut hitstop: MessageWriter<RequestHitstop>,
    mut flash: MessageWriter<RequestFlash>,
    mut squash: MessageWriter<RequestSquashStretch>,
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
    let Ok(target) = target.single() else {
        return;
    };

    trauma.write(AddTrauma {
        target: ListenerTarget::Entity(camera),
        trauma: 0.32 * pane.trauma_scale,
        origin: None,
        attenuation: None,
        propagation_speed: None,
        directional_bias: Vec3::new(0.10, -0.02, 0.0) * pane.impulse_scale,
        profile_override: None,
    });
    hitstop.write(RequestHitstop {
        target: TimeScaleTarget::World,
        hold_frames: pane.hitstop_hold_frames.round().max(0.0) as u32,
        recovery_frames: pane.hitstop_recovery_frames.round().max(0.0) as u32,
        ..RequestHitstop::new(TimeScaleTarget::World, 3)
    });
    flash.write(RequestFlash {
        target: FlashTarget::EntityAndScreen {
            entity: target,
            screen: ListenerTarget::Entity(camera),
        },
        color: Color::WHITE,
        intensity: 0.45 * pane.flash_scale,
        chromatic_aberration: 0.05 * pane.chromatic_scale,
        vignette: 0.12 * pane.vignette_scale,
        ..RequestFlash::new(FlashTarget::Entity(target), Color::WHITE, 0.45)
    });
    squash.write(RequestSquashStretch::new(
        target,
        Vec3::splat(1.0).lerp(
            Vec3::new(1.18, 0.82, 1.0),
            pane.impulse_scale.clamp(0.0, 2.0),
        ),
    ));
}
