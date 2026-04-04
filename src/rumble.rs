use crate::{
    channels::GameFeelChannels,
    config::{EffectTimeDomain, GameFeelDiagnostics},
    time_scale::GlobalTimeScale,
    tween::Tween,
};
use bevy::prelude::*;

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct RumbleListener {
    pub channels: GameFeelChannels,
    pub low_frequency_scale: f32,
    pub high_frequency_scale: f32,
}

impl Default for RumbleListener {
    fn default() -> Self {
        Self {
            channels: GameFeelChannels::default(),
            low_frequency_scale: 1.0,
            high_frequency_scale: 1.0,
        }
    }
}

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct RumbleOutput {
    pub low_frequency: f32,
    pub high_frequency: f32,
}

impl Default for RumbleOutput {
    fn default() -> Self {
        Self {
            low_frequency: 0.0,
            high_frequency: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ActiveRumblePulse {
    pub low_frequency: f32,
    pub high_frequency: f32,
    pub tween: Tween,
    pub clock: EffectTimeDomain,
    pub elapsed_secs: f32,
}

#[derive(Component, Debug, Default)]
pub(crate) struct RumbleRuntime {
    pub effects: Vec<ActiveRumblePulse>,
}

pub(crate) fn update_rumble_outputs(
    global: Res<GlobalTimeScale>,
    mut diagnostics: ResMut<GameFeelDiagnostics>,
    mut query: Query<(&RumbleListener, &mut RumbleOutput, &mut RumbleRuntime)>,
) {
    diagnostics.active_rumble_listeners = 0;

    for (listener, mut output, mut runtime) in &mut query {
        let mut strongest = RumbleOutput::default();
        runtime.effects.retain_mut(|effect| {
            let sample = effect.tween.sample(effect.elapsed_secs);
            let weight = 1.0 - sample.eased;
            strongest.low_frequency = strongest
                .low_frequency
                .max(effect.low_frequency * weight * listener.low_frequency_scale);
            strongest.high_frequency = strongest
                .high_frequency
                .max(effect.high_frequency * weight * listener.high_frequency_scale);
            effect.elapsed_secs += match effect.clock {
                EffectTimeDomain::Unscaled => global.unscaled_delta_secs,
                EffectTimeDomain::GlobalScaled => global.scaled_delta_secs,
            };
            !effect.tween.sample(effect.elapsed_secs).finished
        });

        if runtime.effects.is_empty() {
            *output = RumbleOutput::default();
            continue;
        }

        diagnostics.active_rumble_listeners += 1;

        *output = strongest;
    }
}

#[cfg(test)]
#[path = "rumble_tests.rs"]
mod rumble_tests;
