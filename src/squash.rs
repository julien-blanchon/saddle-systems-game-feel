use crate::{
    config::{EffectTimeDomain, GameFeelDiagnostics},
    time_scale::GlobalTimeScale,
    tween::Tween,
};
use bevy::{math::curve::easing::EaseFunction, prelude::*};

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ScaleEffectMode {
    #[default]
    Relative,
    Absolute,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ScaleStacking {
    #[default]
    Multiply,
    Strongest,
    Replace,
}

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct SquashStretchState {
    pub scale_multiplier: Vec3,
}

impl Default for SquashStretchState {
    fn default() -> Self {
        Self {
            scale_multiplier: Vec3::ONE,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ActiveScaleEffect {
    pub peak_scale: Vec3,
    pub mode: ScaleEffectMode,
    pub tween: Tween,
    pub clock: EffectTimeDomain,
    pub elapsed_secs: f32,
}

#[derive(Component, Debug, Default)]
pub(crate) struct ScaleEffectRuntime {
    pub stacking: ScaleStacking,
    pub effects: Vec<ActiveScaleEffect>,
}

pub(crate) fn update_scale_outputs(
    global: Res<GlobalTimeScale>,
    mut diagnostics: ResMut<GameFeelDiagnostics>,
    mut query: Query<(&mut SquashStretchState, &mut ScaleEffectRuntime)>,
) {
    diagnostics.active_scale_effects = 0;

    for (mut state, mut runtime) in &mut query {
        runtime.effects.retain_mut(|effect| {
            effect.elapsed_secs += match effect.clock {
                EffectTimeDomain::Unscaled => global.unscaled_delta_secs,
                EffectTimeDomain::GlobalScaled => global.scaled_delta_secs,
            };
            !effect.tween.sample(effect.elapsed_secs).finished
        });

        if runtime.effects.is_empty() {
            state.scale_multiplier = Vec3::ONE;
            continue;
        }

        diagnostics.active_scale_effects += runtime.effects.len();

        state.scale_multiplier = match runtime.stacking {
            ScaleStacking::Multiply => runtime
                .effects
                .iter()
                .fold(Vec3::ONE, |acc, effect| acc * sample_scale(effect)),
            ScaleStacking::Strongest => runtime
                .effects
                .iter()
                .max_by(|left, right| {
                    strongest_delta(sample_scale(left))
                        .partial_cmp(&strongest_delta(sample_scale(right)))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map_or(Vec3::ONE, sample_scale),
            ScaleStacking::Replace => runtime.effects.last().map_or(Vec3::ONE, sample_scale),
        };
    }
}

#[must_use]
fn sample_scale(effect: &ActiveScaleEffect) -> Vec3 {
    let sample = effect.tween.sample(effect.elapsed_secs);
    let eased = sample.eased;
    match effect.mode {
        ScaleEffectMode::Relative => Vec3::ONE + (effect.peak_scale - Vec3::ONE) * eased,
        ScaleEffectMode::Absolute => effect.peak_scale.lerp(Vec3::ONE, 1.0 - eased),
    }
}

fn strongest_delta(scale: Vec3) -> f32 {
    let delta = scale - Vec3::ONE;
    delta.x.abs().max(delta.y.abs()).max(delta.z.abs())
}

pub(crate) fn directional_scale(direction: Vec3, magnitude: f32) -> Vec3 {
    if direction.length_squared() <= f32::EPSILON || magnitude.abs() <= f32::EPSILON {
        return Vec3::ONE;
    }

    let dir = direction.abs().normalize_or_zero();
    let stretch = Vec3::ONE + dir * magnitude.max(0.0);
    let squeeze = Vec3::ONE - (Vec3::ONE - dir) * (magnitude.abs() * 0.5);
    Vec3::new(
        if dir.x > 0.01 { stretch.x } else { squeeze.x },
        if dir.y > 0.01 { stretch.y } else { squeeze.y },
        if dir.z > 0.01 { stretch.z } else { squeeze.z },
    )
}

pub(crate) fn default_scale_tween(duration_secs: f32, easing: EaseFunction) -> Tween {
    let leg_duration = (duration_secs.max(f32::EPSILON) * 0.5).max(f32::EPSILON);
    Tween {
        delay_secs: 0.0,
        duration_secs: leg_duration,
        easing,
        repeat: crate::tween::TweenRepeat::PingPong { loops: 1 },
    }
}

#[cfg(test)]
#[path = "squash_tests.rs"]
mod tests;
