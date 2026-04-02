use super::*;

#[test]
fn decay_to_zero_clamps_at_zero() {
    assert!((decay_to_zero(0.8, 1.0, 0.25) - 0.55).abs() < 0.000_1);
    assert_eq!(decay_to_zero(0.1, 10.0, 0.25), 0.0);
}

#[test]
fn trauma_power_curve_is_exponential_and_clamped() {
    assert!((trauma_shake_amount(0.5, 2.0) - 0.25).abs() < 0.000_1);
    assert!((trauma_shake_amount(1.5, 2.0) - 1.0).abs() < 0.000_1);
    assert_eq!(trauma_shake_amount(-0.2, 2.0), 0.0);
}

#[test]
fn attenuation_falls_off_outside_inner_radius() {
    let attenuation = DistanceAttenuation {
        inner_radius: 2.0,
        outer_radius: 10.0,
        exponent: 1.0,
    };

    assert!((attenuated_trauma(0.8, Some(attenuation), 1.0) - 0.8).abs() < 0.000_1);
    assert!(attenuated_trauma(0.8, Some(attenuation), 12.0) <= 0.000_1);
}

#[test]
fn propagation_delay_uses_distance_over_speed() {
    assert!((propagation_delay_secs(Some(12.0), 6.0) - 0.5).abs() < 0.000_1);
    assert_eq!(propagation_delay_secs(None, 6.0), 0.0);
    assert_eq!(propagation_delay_secs(Some(0.0), 6.0), 0.0);
}

#[test]
fn shake_budget_limits_accepted_trauma_per_frame() {
    let listener = ShakeListener::default();
    let mut state = ShakeState::default();
    let mut runtime = ShakeRuntime {
        seeds: [0.0; 6],
        frame_budget_used: 0.0,
    };

    accept_shake(
        &listener,
        &listener.profile,
        &mut state,
        &mut runtime,
        0.9,
        Vec3::X,
    );

    assert!(state.trauma <= listener.profile.budget.max_trauma_per_frame + 0.000_1);
    let first_frame_trauma = state.trauma;
    accept_shake(
        &listener,
        &listener.profile,
        &mut state,
        &mut runtime,
        0.4,
        Vec3::Y,
    );
    assert!((state.trauma - first_frame_trauma).abs() < 0.000_1);
}
