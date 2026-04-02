use super::*;
use bevy::math::curve::easing::EaseFunction;

#[test]
fn once_tween_respects_delay_and_completion() {
    let tween = Tween {
        delay_secs: 0.1,
        duration_secs: 0.2,
        easing: EaseFunction::Linear,
        repeat: TweenRepeat::Once,
    };

    let before_start = tween.sample(0.05);
    assert!(!before_start.active);
    assert!(!before_start.finished);

    let midway = tween.sample(0.20);
    assert!(midway.active);
    assert!(!midway.finished);
    assert!((midway.progress - 0.5).abs() < 0.000_1);

    let done = tween.sample(0.35);
    assert!(done.finished);
    assert!((done.eased - 1.0).abs() < 0.000_1);
}

#[test]
fn ping_pong_tween_reports_expected_total_duration() {
    let tween = Tween {
        delay_secs: 0.0,
        duration_secs: 0.15,
        easing: EaseFunction::Linear,
        repeat: TweenRepeat::PingPong { loops: 1 },
    };

    assert!((tween.total_duration_secs() - 0.30).abs() < 0.000_1);
    assert!(tween.sample(0.075).progress > 0.45);
    assert!(tween.sample(0.225).progress < 0.55);
}

#[test]
fn attack_sustain_decay_returns_to_zero_after_decay() {
    let envelope = AttackSustainDecay {
        attack_secs: 0.1,
        sustain_secs: 0.1,
        decay_secs: 0.2,
        attack_easing: EaseFunction::Linear,
        decay_easing: EaseFunction::Linear,
    };

    assert_eq!(envelope.sample(0.0), 0.0);
    assert!((envelope.sample(0.05) - 0.5).abs() < 0.000_1);
    assert!((envelope.sample(0.15) - 1.0).abs() < 0.000_1);
    assert!((envelope.sample(0.30) - 0.5).abs() < 0.000_1);
    assert_eq!(envelope.sample(0.45), 0.0);
}
