use super::*;

#[test]
fn directional_scale_stretches_along_motion_axis() {
    let scale = directional_scale(Vec3::X, 0.4);
    assert!(scale.x > 1.3);
    assert!(scale.y < 1.0);
    assert!(scale.z < 1.0);
}

#[test]
fn default_scale_tween_uses_full_requested_duration() {
    let tween = default_scale_tween(0.24, bevy::math::curve::easing::EaseFunction::Linear);
    assert!((tween.total_duration_secs() - 0.24).abs() < 0.000_1);
}

#[test]
fn strongest_delta_prefers_largest_axis_change() {
    assert!(strongest_delta(Vec3::new(1.3, 0.9, 1.0)) > strongest_delta(Vec3::new(1.1, 0.95, 1.0)));
}
