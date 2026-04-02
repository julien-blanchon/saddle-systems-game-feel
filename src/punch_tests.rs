use super::*;

#[test]
fn spring_step_converges_back_to_origin() {
    let mut offset = Vec3::new(1.0, 0.0, 0.0);
    let mut velocity = Vec3::ZERO;
    for _ in 0..120 {
        step_spring_vec3(
            &mut offset,
            &mut velocity,
            SpringSettings::critically_damped(7.5),
            1.0 / 60.0,
        );
    }

    assert!(offset.length() < 0.02);
    assert!(velocity.length() < 0.1);
}

#[test]
fn world_space_punch_rotates_into_listener_local_space() {
    let transform = Transform::from_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
    let event = PendingPunchEvent {
        listener: Entity::PLACEHOLDER,
        translation: Vec3::new(1.0, 0.0, 0.0),
        rotation: Vec3::new(0.0, 0.5, 0.0),
        fov: 0.0,
        space: ImpulseSpace::World,
        profile: PunchProfile::default(),
        due_in_secs: 0.0,
    };
    let mut state = PunchState::default();
    let mut runtime = PunchRuntime::default();

    inject_punch(&transform, &event, 1.0, &mut state, &mut runtime);

    assert!((runtime.world_translation_velocity - Vec3::X).length() < 0.000_1);
    assert!(
        (runtime.rotation_velocity - (transform.rotation.inverse() * event.rotation)).length()
            < 0.000_1
    );
}

#[test]
fn attenuation_and_delay_scale_with_distance() {
    let falloff = DistanceAttenuation {
        inner_radius: 0.0,
        outer_radius: 10.0,
        exponent: 1.0,
    };
    assert!((attenuation(2.0, Some(falloff), 5.0) - 1.0).abs() < 0.000_1);
    assert!((propagation_delay(Some(20.0), 10.0) - 0.5).abs() < 0.000_1);
}
