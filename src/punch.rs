use crate::{
    channels::GameFeelChannels,
    config::{DistanceAttenuation, EffectTimeDomain, GameFeelDiagnostics},
    time_scale::GlobalTimeScale,
};
use bevy::prelude::*;

#[derive(Reflect, Clone, Copy, Debug, PartialEq)]
pub struct SpringSettings {
    pub frequency_hz: f32,
    pub damping_ratio: f32,
}

impl SpringSettings {
    #[must_use]
    pub fn critically_damped(frequency_hz: f32) -> Self {
        Self {
            frequency_hz,
            damping_ratio: 1.0,
        }
    }
}

impl Default for SpringSettings {
    fn default() -> Self {
        Self::critically_damped(7.5)
    }
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct PunchProfile {
    pub translation_gain: Vec3,
    pub rotation_gain: Vec3,
    pub fov_gain: f32,
    pub spring: SpringSettings,
    pub time_domain: EffectTimeDomain,
}

impl Default for PunchProfile {
    fn default() -> Self {
        Self {
            translation_gain: Vec3::ONE,
            rotation_gain: Vec3::ONE,
            fov_gain: 1.0,
            spring: SpringSettings::default(),
            time_domain: EffectTimeDomain::Unscaled,
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct PunchListener {
    pub channels: GameFeelChannels,
    pub accessibility_scale: f32,
    pub profile: PunchProfile,
}

impl Default for PunchListener {
    fn default() -> Self {
        Self {
            channels: GameFeelChannels::default(),
            accessibility_scale: 1.0,
            profile: PunchProfile::default(),
        }
    }
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ImpulseSpace {
    #[default]
    Local,
    World,
}

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct PunchState {
    pub local_translation_offset: Vec3,
    pub world_translation_offset: Vec3,
    pub rotation_offset: Vec3,
    pub fov_offset: f32,
}

impl Default for PunchState {
    fn default() -> Self {
        Self {
            local_translation_offset: Vec3::ZERO,
            world_translation_offset: Vec3::ZERO,
            rotation_offset: Vec3::ZERO,
            fov_offset: 0.0,
        }
    }
}

#[derive(Component, Debug, Default)]
pub(crate) struct PunchRuntime {
    pub local_translation_velocity: Vec3,
    pub world_translation_velocity: Vec3,
    pub rotation_velocity: Vec3,
    pub fov_velocity: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct PendingPunchEvent {
    pub listener: Entity,
    pub translation: Vec3,
    pub rotation: Vec3,
    pub fov: f32,
    pub space: ImpulseSpace,
    pub profile: PunchProfile,
    pub due_in_secs: f32,
}

#[derive(Resource, Debug, Default)]
pub(crate) struct PendingPunchQueue {
    pub events: Vec<PendingPunchEvent>,
}

pub(crate) fn advance_punch_queue(
    global: Res<GlobalTimeScale>,
    mut queue: ResMut<PendingPunchQueue>,
) {
    for event in &mut queue.events {
        event.due_in_secs -= global.unscaled_delta_secs;
    }
}

pub(crate) fn apply_due_punch_events(
    mut commands: Commands,
    mut queue: ResMut<PendingPunchQueue>,
    mut query: Query<
        (
            &PunchListener,
            &Transform,
            Option<&mut PunchState>,
            Option<&mut PunchRuntime>,
        ),
        With<PunchListener>,
    >,
) {
    let mut still_waiting = Vec::with_capacity(queue.events.len());

    for event in queue.events.drain(..) {
        if event.due_in_secs > 0.0 {
            still_waiting.push(event);
            continue;
        }

        let Ok((listener, transform, state, runtime)) = query.get_mut(event.listener) else {
            continue;
        };
        let accessibility = listener.accessibility_scale.clamp(0.0, 2.0);

        match (state, runtime) {
            (Some(mut state), Some(mut runtime)) => {
                inject_punch(transform, &event, accessibility, &mut state, &mut runtime);
            }
            _ => {
                let mut state = PunchState::default();
                let mut runtime = PunchRuntime::default();
                inject_punch(transform, &event, accessibility, &mut state, &mut runtime);
                commands.entity(event.listener).insert((state, runtime));
            }
        }
    }

    queue.events = still_waiting;
}

pub(crate) fn update_punch_states(
    global: Res<GlobalTimeScale>,
    mut diagnostics: ResMut<GameFeelDiagnostics>,
    mut query: Query<(&PunchListener, &mut PunchState, &mut PunchRuntime)>,
) {
    diagnostics.active_punch_listeners = 0;

    for (listener, mut state, mut runtime) in &mut query {
        let delta = match listener.profile.time_domain {
            EffectTimeDomain::Unscaled => global.unscaled_delta_secs,
            EffectTimeDomain::GlobalScaled => global.scaled_delta_secs,
        };
        if delta <= 0.0 {
            continue;
        }

        let spring = listener.profile.spring;
        step_spring_vec3(
            &mut state.local_translation_offset,
            &mut runtime.local_translation_velocity,
            spring,
            delta,
        );
        step_spring_vec3(
            &mut state.world_translation_offset,
            &mut runtime.world_translation_velocity,
            spring,
            delta,
        );
        step_spring_vec3(
            &mut state.rotation_offset,
            &mut runtime.rotation_velocity,
            spring,
            delta,
        );
        step_spring_scalar(
            &mut state.fov_offset,
            &mut runtime.fov_velocity,
            spring,
            delta,
        );

        let active = state.local_translation_offset.length_squared() > 1.0e-6
            || state.world_translation_offset.length_squared() > 1.0e-6
            || state.rotation_offset.length_squared() > 1.0e-6
            || state.fov_offset.abs() > 1.0e-6
            || runtime.local_translation_velocity.length_squared() > 1.0e-6
            || runtime.world_translation_velocity.length_squared() > 1.0e-6
            || runtime.rotation_velocity.length_squared() > 1.0e-6
            || runtime.fov_velocity.abs() > 1.0e-6;
        if active {
            diagnostics.active_punch_listeners += 1;
        } else {
            state.local_translation_offset = Vec3::ZERO;
            state.world_translation_offset = Vec3::ZERO;
            state.rotation_offset = Vec3::ZERO;
            state.fov_offset = 0.0;
            runtime.local_translation_velocity = Vec3::ZERO;
            runtime.world_translation_velocity = Vec3::ZERO;
            runtime.rotation_velocity = Vec3::ZERO;
            runtime.fov_velocity = 0.0;
        }
    }
}

pub(crate) fn queue_punch_event(
    queue: &mut PendingPunchQueue,
    listener: Entity,
    translation: Vec3,
    rotation: Vec3,
    fov: f32,
    space: ImpulseSpace,
    profile: PunchProfile,
    due_in_secs: f32,
) {
    queue.events.push(PendingPunchEvent {
        listener,
        translation,
        rotation,
        fov,
        space,
        profile,
        due_in_secs,
    });
}

#[must_use]
pub(crate) fn attenuation(
    magnitude: f32,
    attenuation: Option<DistanceAttenuation>,
    distance: f32,
) -> f32 {
    magnitude * attenuation.map_or(1.0, |attenuation| attenuation.sample(distance))
}

#[must_use]
pub(crate) fn propagation_delay(propagation_speed: Option<f32>, distance: f32) -> f32 {
    let Some(speed) = propagation_speed else {
        return 0.0;
    };
    if speed <= f32::EPSILON {
        return 0.0;
    }
    distance / speed
}

fn inject_punch(
    transform: &Transform,
    event: &PendingPunchEvent,
    accessibility_scale: f32,
    state: &mut PunchState,
    runtime: &mut PunchRuntime,
) {
    let translation = event.translation * event.profile.translation_gain * accessibility_scale;
    let rotation = event.rotation * event.profile.rotation_gain * accessibility_scale;
    let fov = event.fov * event.profile.fov_gain * accessibility_scale;

    match event.space {
        ImpulseSpace::Local => {
            runtime.local_translation_velocity += translation;
            runtime.rotation_velocity += rotation;
        }
        ImpulseSpace::World => {
            runtime.world_translation_velocity += translation;
            runtime.rotation_velocity += transform.rotation.inverse() * rotation;
        }
    }
    runtime.fov_velocity += fov;
    let _ = state;
}

pub(crate) fn step_spring_vec3(
    offset: &mut Vec3,
    velocity: &mut Vec3,
    spring: SpringSettings,
    delta_secs: f32,
) {
    let angular_frequency = spring.frequency_hz.max(0.01) * std::f32::consts::TAU;
    let damping = 2.0 * spring.damping_ratio.max(0.0) * angular_frequency;
    let acceleration = -angular_frequency.powi(2) * *offset - damping * *velocity;
    *velocity += acceleration * delta_secs;
    *offset += *velocity * delta_secs;
}

pub(crate) fn step_spring_scalar(
    offset: &mut f32,
    velocity: &mut f32,
    spring: SpringSettings,
    delta_secs: f32,
) {
    let angular_frequency = spring.frequency_hz.max(0.01) * std::f32::consts::TAU;
    let damping = 2.0 * spring.damping_ratio.max(0.0) * angular_frequency;
    let acceleration = -angular_frequency.powi(2) * *offset - damping * *velocity;
    *velocity += acceleration * delta_secs;
    *offset += *velocity * delta_secs;
}

#[cfg(test)]
#[path = "punch_tests.rs"]
mod tests;
