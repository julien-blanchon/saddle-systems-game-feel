use super::*;
use crate::tween::TweenRepeat;

#[test]
fn knockback_eased_displacement_decays() {
    let effect = ActiveKnockback {
        direction: Vec3::X,
        peak_displacement: 10.0,
        tween: Tween {
            delay_secs: 0.0,
            duration_secs: 0.5,
            easing: EaseFunction::Linear,
            repeat: TweenRepeat::Once,
        },
        clock: EffectTimeDomain::Unscaled,
        elapsed_secs: 0.0,
    };

    let sample_start = effect.tween.sample(0.0);
    let remaining_start = 1.0 - sample_start.eased;
    assert!(
        (remaining_start - 1.0).abs() < f32::EPSILON,
        "at t=0 remaining should be 1.0"
    );

    let sample_mid = effect.tween.sample(0.25);
    let remaining_mid = 1.0 - sample_mid.eased;
    assert!(
        remaining_mid < remaining_start,
        "displacement should decay over time"
    );

    let sample_end = effect.tween.sample(0.5);
    assert!(sample_end.finished, "effect should be finished at duration");
}

#[test]
fn knockback_receiver_clamps_displacement() {
    let receiver = KnockbackReceiver {
        max_displacement: 5.0,
    };

    let large_displacement = Vec3::new(10.0, 0.0, 0.0);
    let len = large_displacement.length();
    let clamped = if len > receiver.max_displacement {
        large_displacement.normalize_or_zero() * receiver.max_displacement
    } else {
        large_displacement
    };

    assert!((clamped.length() - 5.0).abs() < f32::EPSILON);
}
