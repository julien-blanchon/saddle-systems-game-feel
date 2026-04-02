use super::*;

#[test]
fn resolve_effective_time_scale_honors_ignore_flags() {
    let global = GlobalTimeScale {
        base_scale: 0.8,
        hitstop_scale: 0.5,
        scale: 0.4,
        ..default()
    };
    let local = EntityTimeScale {
        base_scale: 0.75,
        hitstop_scale: 0.5,
        scale: 0.375,
    };

    assert!(
        (resolve_effective_time_scale(&global, Some(&local), false, false) - 0.15).abs() < 0.000_1
    );
    assert!(
        (resolve_effective_time_scale(&global, Some(&local), true, false) - 0.375).abs() < 0.000_1
    );
    assert!(
        (resolve_effective_time_scale(&global, Some(&local), false, true) - 0.6).abs() < 0.000_1
    );
}

#[test]
fn equal_priority_ramps_pick_the_lowest_scale() {
    let ramps = [
        TimeScaleRamp {
            scale: 0.75,
            ramp_in_secs: 0.0,
            hold_secs: 1.0,
            ramp_out_secs: 0.0,
            easing: bevy::math::curve::easing::EaseFunction::Linear,
            priority: 2,
            elapsed_secs: 0.1,
        },
        TimeScaleRamp {
            scale: 0.55,
            ramp_in_secs: 0.0,
            hold_secs: 1.0,
            ramp_out_secs: 0.0,
            easing: bevy::math::curve::easing::EaseFunction::Linear,
            priority: 2,
            elapsed_secs: 0.1,
        },
    ];

    assert!((resolve_ramp_scale(&ramps) - 0.55).abs() < 0.000_1);
}

#[test]
fn additive_hitstop_caps_frames() {
    let mut slot = Some(ActiveHitstop {
        hold_frames_remaining: 3,
        recovery_frames_total: 2,
        recovery_frames_elapsed: 0,
    });

    apply_hitstop_stacking(
        &mut slot,
        4,
        1,
        HitstopStacking::AdditiveWithCap { frame_cap: 5 },
    );

    let hitstop = slot.expect("hitstop should still exist");
    assert_eq!(hitstop.hold_frames_remaining, 5);
    assert_eq!(hitstop.recovery_frames_total, 2);
}
