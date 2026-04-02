use crate::{
    channels::GameFeelChannels,
    config::{DistanceAttenuation, EffectTimeDomain, GameFeelDiagnostics},
    time_scale::GlobalTimeScale,
};
use bevy::{math::StableInterpolate, prelude::*};

#[derive(Resource, Reflect, Clone, Copy, Debug, PartialEq)]
#[reflect(Resource, Default)]
pub struct ShakeAccessibility {
    pub multiplier: f32,
}

impl Default for ShakeAccessibility {
    fn default() -> Self {
        Self { multiplier: 1.0 }
    }
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct ShakeBudget {
    pub max_trauma_per_frame: f32,
    pub fatigue_gain_per_trauma: f32,
    pub fatigue_decay_per_second: f32,
    pub minimum_acceptance: f32,
}

impl Default for ShakeBudget {
    fn default() -> Self {
        Self {
            max_trauma_per_frame: 0.65,
            fatigue_gain_per_trauma: 0.85,
            fatigue_decay_per_second: 1.8,
            minimum_acceptance: 0.15,
        }
    }
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct ShakeProfile {
    pub translation_amplitude: Vec3,
    pub rotation_amplitude: Vec3,
    pub frequency_hz: f32,
    pub decay_per_second: f32,
    pub trauma_power: f32,
    pub directional_bias_decay_per_second: f32,
    pub budget: ShakeBudget,
    pub time_domain: EffectTimeDomain,
}

impl Default for ShakeProfile {
    fn default() -> Self {
        Self {
            translation_amplitude: Vec3::new(0.25, 0.25, 0.10),
            rotation_amplitude: Vec3::new(0.04, 0.04, 0.07),
            frequency_hz: 18.0,
            decay_per_second: 1.8,
            trauma_power: 2.0,
            directional_bias_decay_per_second: 8.0,
            budget: ShakeBudget::default(),
            time_domain: EffectTimeDomain::Unscaled,
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct ShakeListener {
    pub channels: GameFeelChannels,
    pub accessibility_scale: f32,
    pub profile: ShakeProfile,
}

impl Default for ShakeListener {
    fn default() -> Self {
        Self {
            channels: GameFeelChannels::default(),
            accessibility_scale: 1.0,
            profile: ShakeProfile::default(),
        }
    }
}

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct ShakeState {
    pub trauma: f32,
    pub fatigue: f32,
    pub directional_bias: Vec3,
    pub translation_offset: Vec3,
    pub rotation_offset: Vec3,
}

impl Default for ShakeState {
    fn default() -> Self {
        Self {
            trauma: 0.0,
            fatigue: 0.0,
            directional_bias: Vec3::ZERO,
            translation_offset: Vec3::ZERO,
            rotation_offset: Vec3::ZERO,
        }
    }
}

#[derive(Component, Debug)]
pub(crate) struct ShakeRuntime {
    pub seeds: [f32; 6],
    pub frame_budget_used: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct PendingShakeEvent {
    pub listener: Entity,
    pub trauma: f32,
    pub directional_bias: Vec3,
    pub profile: ShakeProfile,
    pub due_in_secs: f32,
}

#[derive(Resource, Debug, Default)]
pub(crate) struct PendingShakeQueue {
    pub events: Vec<PendingShakeEvent>,
}

pub(crate) fn advance_shake_queue(
    global: Res<GlobalTimeScale>,
    mut queue: ResMut<PendingShakeQueue>,
) {
    for event in &mut queue.events {
        event.due_in_secs -= global.unscaled_delta_secs;
    }
}

pub(crate) fn prepare_shake_states(
    global: Res<GlobalTimeScale>,
    mut query: Query<(&ShakeListener, &mut ShakeState, &mut ShakeRuntime)>,
) {
    for (listener, mut state, mut runtime) in &mut query {
        let delta = delta_for(listener.profile.time_domain, &global);
        state.trauma = decay_to_zero(state.trauma, listener.profile.decay_per_second, delta);
        state.fatigue = decay_to_zero(
            state.fatigue,
            listener.profile.budget.fatigue_decay_per_second,
            global.unscaled_delta_secs,
        );
        state.directional_bias = state.directional_bias.lerp(
            Vec3::ZERO,
            (listener.profile.directional_bias_decay_per_second * delta).clamp(0.0, 1.0),
        );
        runtime.frame_budget_used = 0.0;
    }
}

pub(crate) fn apply_due_shake_events(
    mut commands: Commands,
    mut queue: ResMut<PendingShakeQueue>,
    mut query: Query<
        (
            &ShakeListener,
            Option<&mut ShakeState>,
            Option<&mut ShakeRuntime>,
        ),
        With<ShakeListener>,
    >,
) {
    let mut still_waiting = Vec::with_capacity(queue.events.len());
    for event in queue.events.drain(..) {
        if event.due_in_secs > 0.0 {
            still_waiting.push(event);
            continue;
        }

        let Ok((listener, state, runtime)) = query.get_mut(event.listener) else {
            continue;
        };

        match (state, runtime) {
            (Some(mut state), Some(mut runtime)) => {
                accept_shake(
                    listener,
                    &event.profile,
                    &mut state,
                    &mut runtime,
                    event.trauma,
                    event.directional_bias,
                );
            }
            _ => {
                let mut state = ShakeState::default();
                let mut runtime = ShakeRuntime {
                    seeds: seed_set(event.listener),
                    frame_budget_used: 0.0,
                };
                accept_shake(
                    listener,
                    &event.profile,
                    &mut state,
                    &mut runtime,
                    event.trauma,
                    event.directional_bias,
                );
                commands.entity(event.listener).insert((state, runtime));
            }
        }
    }
    queue.events = still_waiting;
}

pub(crate) fn update_shake_outputs(
    global: Res<GlobalTimeScale>,
    accessibility: Res<ShakeAccessibility>,
    mut diagnostics: ResMut<GameFeelDiagnostics>,
    mut query: Query<(&ShakeListener, &mut ShakeState, &ShakeRuntime)>,
) {
    diagnostics.active_shake_listeners = 0;

    for (listener, mut state, runtime) in &mut query {
        let shake_amount = trauma_shake_amount(state.trauma, listener.profile.trauma_power);
        if shake_amount <= 0.0001 {
            state.translation_offset = Vec3::ZERO;
            state.rotation_offset = Vec3::ZERO;
            continue;
        }

        diagnostics.active_shake_listeners += 1;

        let domain_time = time_for(listener.profile.time_domain, &global);
        let sample_time = domain_time * listener.profile.frequency_hz;
        let gain = (listener.accessibility_scale * accessibility.multiplier).clamp(0.0, 2.0);
        if gain <= f32::EPSILON {
            state.translation_offset = Vec3::ZERO;
            state.rotation_offset = Vec3::ZERO;
            continue;
        }

        let translation_noise = Vec3::new(
            perlin(sample_time + runtime.seeds[0]),
            perlin(sample_time + runtime.seeds[1]),
            perlin(sample_time + runtime.seeds[2]),
        );
        let rotation_noise = Vec3::new(
            perlin(sample_time + runtime.seeds[3]),
            perlin(sample_time + runtime.seeds[4]),
            perlin(sample_time + runtime.seeds[5]),
        );

        state.translation_offset = (translation_noise * listener.profile.translation_amplitude
            + state.directional_bias)
            * shake_amount
            * gain;
        state.rotation_offset =
            rotation_noise * listener.profile.rotation_amplitude * shake_amount * gain;
    }
}

pub(crate) fn queue_shake_event(
    queue: &mut PendingShakeQueue,
    listener: Entity,
    trauma: f32,
    directional_bias: Vec3,
    profile: ShakeProfile,
    due_in_secs: f32,
) {
    queue.events.push(PendingShakeEvent {
        listener,
        trauma,
        directional_bias,
        profile,
        due_in_secs,
    });
}

#[must_use]
pub(crate) fn attenuated_trauma(
    trauma: f32,
    attenuation: Option<DistanceAttenuation>,
    distance: f32,
) -> f32 {
    trauma * attenuation.map_or(1.0, |attenuation| attenuation.sample(distance))
}

#[must_use]
pub(crate) fn propagation_delay_secs(propagation_speed: Option<f32>, distance: f32) -> f32 {
    let Some(speed) = propagation_speed else {
        return 0.0;
    };
    if speed <= f32::EPSILON {
        return 0.0;
    }
    distance / speed
}

fn accept_shake(
    listener: &ShakeListener,
    profile: &ShakeProfile,
    state: &mut ShakeState,
    runtime: &mut ShakeRuntime,
    requested_trauma: f32,
    directional_bias: Vec3,
) {
    let remaining_budget =
        (profile.budget.max_trauma_per_frame - runtime.frame_budget_used).max(0.0);
    let fatigue_factor = (1.0 / (1.0 + state.fatigue)).max(profile.budget.minimum_acceptance);
    let accepted = (requested_trauma * fatigue_factor)
        .min(remaining_budget)
        .clamp(0.0, 1.0);
    if accepted <= 0.0 {
        return;
    }

    runtime.frame_budget_used += accepted;
    state.trauma = (state.trauma + accepted).clamp(0.0, 1.0);
    state.fatigue += accepted * profile.budget.fatigue_gain_per_trauma;
    state.directional_bias += directional_bias * accepted;

    if state.directional_bias.length_squared() > 4.0 {
        state.directional_bias = state.directional_bias.normalize() * 2.0;
    }

    let _ = listener;
}

#[must_use]
pub(crate) fn decay_to_zero(value: f32, rate_per_second: f32, delta_secs: f32) -> f32 {
    (value - rate_per_second.max(0.0) * delta_secs.max(0.0)).max(0.0)
}

#[must_use]
pub(crate) fn trauma_shake_amount(trauma: f32, trauma_power: f32) -> f32 {
    trauma.clamp(0.0, 1.0).powf(trauma_power.max(0.01))
}

#[must_use]
fn delta_for(domain: EffectTimeDomain, global: &GlobalTimeScale) -> f32 {
    match domain {
        EffectTimeDomain::Unscaled => global.unscaled_delta_secs,
        EffectTimeDomain::GlobalScaled => global.scaled_delta_secs,
    }
}

#[must_use]
fn time_for(domain: EffectTimeDomain, global: &GlobalTimeScale) -> f32 {
    match domain {
        EffectTimeDomain::Unscaled => global.elapsed_unscaled_secs,
        EffectTimeDomain::GlobalScaled => global.elapsed_scaled_secs,
    }
}

#[must_use]
fn seed_set(entity: Entity) -> [f32; 6] {
    let base = entity.to_bits() as f32;
    [
        base * 0.013 + 17.0,
        base * 0.017 + 113.0,
        base * 0.021 + 227.0,
        base * 0.029 + 307.0,
        base * 0.031 + 401.0,
        base * 0.037 + 509.0,
    ]
}

#[must_use]
fn perlin(x: f32) -> f32 {
    let x_floor = x.floor() as usize;
    let xf0 = x - x_floor as f32;
    let xf1 = xf0 - 1.0;
    let xi0 = x_floor & 0xFF;
    let xi1 = (x_floor + 1) & 0xFF;
    let t = fade(xf0).clamp(0.0, 1.0);
    let a = dot_grad(PERMUTATION_TABLE[xi0], xf0);
    let b = dot_grad(PERMUTATION_TABLE[xi1], xf1);
    a.interpolate_stable(&b, t)
}

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn dot_grad(hash: u8, x: f32) -> f32 {
    if hash & 0x1 != 0 { x } else { -x }
}

const PERMUTATION_TABLE: [u8; 256] = [
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69,
    142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219,
    203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175,
    74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133, 230,
    220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73, 209, 76,
    132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173,
    186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206,
    59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44, 154, 163,
    70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232,
    178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162,
    241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204,
    176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141,
    128, 195, 78, 66, 215, 61, 156, 180,
];

#[cfg(test)]
#[path = "shake_tests.rs"]
mod tests;
