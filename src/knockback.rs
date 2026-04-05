use crate::{
    config::{EffectTimeDomain, GameFeelDiagnostics},
    time_scale::GlobalTimeScale,
    tween::Tween,
};
use bevy::prelude::*;

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct KnockbackReceiver {
    pub max_displacement: f32,
}

impl Default for KnockbackReceiver {
    fn default() -> Self {
        Self {
            max_displacement: f32::MAX,
        }
    }
}

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct KnockbackState {
    pub displacement: Vec3,
}

impl Default for KnockbackState {
    fn default() -> Self {
        Self {
            displacement: Vec3::ZERO,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ActiveKnockback {
    pub direction: Vec3,
    pub peak_displacement: f32,
    pub tween: Tween,
    pub clock: EffectTimeDomain,
    pub elapsed_secs: f32,
}

#[derive(Component, Debug, Default)]
pub(crate) struct KnockbackRuntime {
    pub effects: Vec<ActiveKnockback>,
}

pub(crate) fn process_knockback_requests(
    mut requests: MessageReader<crate::messages::RequestKnockback>,
    mut commands: Commands,
    query: Query<(Entity, Option<&KnockbackReceiver>)>,
) {
    for request in requests.read() {
        if query.get(request.target).is_err() {
            continue;
        }

        let tween = Tween {
            delay_secs: 0.0,
            duration_secs: request.duration_secs.max(f32::EPSILON),
            easing: request.easing,
            repeat: crate::tween::TweenRepeat::Once,
        };

        let direction = if request.direction.length_squared() > f32::EPSILON {
            request.direction.normalize_or_zero()
        } else {
            Vec3::ZERO
        };

        let effect = ActiveKnockback {
            direction,
            peak_displacement: request.force,
            tween,
            clock: request.clock,
            elapsed_secs: 0.0,
        };

        commands
            .entity(request.target)
            .queue(move |mut entity: EntityWorldMut<'_>| {
                if let Some(mut runtime) = entity.get_mut::<KnockbackRuntime>() {
                    runtime.effects.push(effect);
                } else {
                    entity.insert((
                        KnockbackRuntime {
                            effects: vec![effect],
                        },
                        KnockbackState::default(),
                    ));
                }
            });
    }
}

pub(crate) fn update_knockback_outputs(
    global: Res<GlobalTimeScale>,
    mut diagnostics: ResMut<GameFeelDiagnostics>,
    mut query: Query<(
        &mut KnockbackState,
        &mut KnockbackRuntime,
        Option<&KnockbackReceiver>,
    )>,
) {
    diagnostics.active_knockback_effects = 0;

    for (mut state, mut runtime, receiver) in &mut query {
        runtime.effects.retain_mut(|effect| {
            effect.elapsed_secs += match effect.clock {
                EffectTimeDomain::Unscaled => global.unscaled_delta_secs,
                EffectTimeDomain::GlobalScaled => global.scaled_delta_secs,
            };
            !effect.tween.sample(effect.elapsed_secs).finished
        });

        if runtime.effects.is_empty() {
            state.displacement = Vec3::ZERO;
            continue;
        }

        diagnostics.active_knockback_effects += runtime.effects.len();

        let mut total = Vec3::ZERO;
        for effect in &runtime.effects {
            let sample = effect.tween.sample(effect.elapsed_secs);
            let remaining = 1.0 - sample.eased;
            total += effect.direction * effect.peak_displacement * remaining;
        }

        if let Some(receiver) = receiver {
            let len = total.length();
            if len > receiver.max_displacement {
                total = total.normalize_or_zero() * receiver.max_displacement;
            }
        }

        state.displacement = total;
    }
}

pub(crate) fn cleanup_knockback(
    mut commands: Commands,
    query: Query<(Entity, &KnockbackRuntime), Without<KnockbackReceiver>>,
) {
    for (entity, runtime) in &query {
        if runtime.effects.is_empty() {
            commands
                .entity(entity)
                .remove::<(KnockbackRuntime, KnockbackState)>();
        }
    }
}

#[cfg(test)]
#[path = "knockback_tests.rs"]
mod tests;
