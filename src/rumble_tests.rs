use super::*;
use crate::{EffectTimeDomain, GameFeelDiagnostics, GlobalTimeScale, Tween, TweenRepeat};
use bevy::math::curve::easing::EaseFunction;

#[test]
fn rumble_outputs_follow_the_strongest_active_effect() {
    let mut app = App::new();
    app.init_resource::<GlobalTimeScale>();
    app.init_resource::<GameFeelDiagnostics>();
    app.add_systems(Update, update_rumble_outputs);

    let entity = app
        .world_mut()
        .spawn((
            RumbleListener::default(),
            RumbleOutput::default(),
            RumbleRuntime {
                effects: vec![
                    ActiveRumblePulse {
                        low_frequency: 0.2,
                        high_frequency: 0.3,
                        tween: Tween {
                            delay_secs: 0.0,
                            duration_secs: 0.5,
                            easing: EaseFunction::Linear,
                            repeat: TweenRepeat::Once,
                        },
                        clock: EffectTimeDomain::Unscaled,
                        elapsed_secs: 0.0,
                    },
                    ActiveRumblePulse {
                        low_frequency: 0.7,
                        high_frequency: 0.4,
                        tween: Tween {
                            delay_secs: 0.0,
                            duration_secs: 0.5,
                            easing: EaseFunction::Linear,
                            repeat: TweenRepeat::Once,
                        },
                        clock: EffectTimeDomain::Unscaled,
                        elapsed_secs: 0.0,
                    },
                ],
            },
        ))
        .id();

    app.world_mut()
        .resource_mut::<GlobalTimeScale>()
        .unscaled_delta_secs = 0.05;
    app.update();

    let output = app.world().get::<RumbleOutput>(entity).unwrap();
    assert!(output.low_frequency >= 0.69);
    assert!(output.high_frequency >= 0.39);
    assert_eq!(
        app.world()
            .resource::<GameFeelDiagnostics>()
            .active_rumble_listeners,
        1
    );
}
