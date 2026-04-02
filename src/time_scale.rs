use crate::config::GameFeelDiagnostics;
use bevy::{math::curve::Curve, prelude::*};

#[derive(Resource, Reflect, Clone, Copy, Debug, PartialEq)]
#[reflect(Resource, Default)]
pub struct GlobalTimeScale {
    pub frame_index: u64,
    pub unscaled_delta_secs: f32,
    pub elapsed_unscaled_secs: f32,
    pub base_scale: f32,
    pub hitstop_scale: f32,
    pub scale: f32,
    pub scaled_delta_secs: f32,
    pub elapsed_scaled_secs: f32,
}

impl Default for GlobalTimeScale {
    fn default() -> Self {
        Self {
            frame_index: 0,
            unscaled_delta_secs: 0.0,
            elapsed_unscaled_secs: 0.0,
            base_scale: 1.0,
            hitstop_scale: 1.0,
            scale: 1.0,
            scaled_delta_secs: 0.0,
            elapsed_scaled_secs: 0.0,
        }
    }
}

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct EntityTimeScale {
    pub base_scale: f32,
    pub hitstop_scale: f32,
    pub scale: f32,
}

impl Default for EntityTimeScale {
    fn default() -> Self {
        Self {
            base_scale: 1.0,
            hitstop_scale: 1.0,
            scale: 1.0,
        }
    }
}

#[derive(Component, Reflect, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[reflect(Component, Default)]
pub struct IgnoreGlobalTimeScale;

#[derive(Component, Reflect, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[reflect(Component, Default)]
pub struct IgnoreHitstop;

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum HitstopStacking {
    #[default]
    Refresh,
    Max,
    AdditiveWithCap {
        frame_cap: u32,
    },
    IgnoreWeaker,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeScaleTarget {
    World,
    Entity(Entity),
    Group,
}

#[derive(Clone, Debug)]
pub(crate) struct TimeScaleRamp {
    pub scale: f32,
    pub ramp_in_secs: f32,
    pub hold_secs: f32,
    pub ramp_out_secs: f32,
    pub easing: bevy::math::curve::easing::EaseFunction,
    pub priority: i32,
    pub elapsed_secs: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ActiveHitstop {
    pub hold_frames_remaining: u32,
    pub recovery_frames_total: u32,
    pub recovery_frames_elapsed: u32,
}

#[derive(Resource, Debug, Default)]
pub(crate) struct WorldTimeScaleRuntime {
    pub ramps: Vec<TimeScaleRamp>,
    pub hitstop: Option<ActiveHitstop>,
}

#[derive(Component, Debug, Default)]
pub(crate) struct LocalTimeScaleRuntime {
    pub ramps: Vec<TimeScaleRamp>,
    pub hitstop: Option<ActiveHitstop>,
}

impl TimeScaleRamp {
    pub fn sample(&self) -> Option<f32> {
        let ramp_in = self.ramp_in_secs.max(0.0);
        let hold = self.hold_secs.max(0.0);
        let ramp_out = self.ramp_out_secs.max(0.0);
        let total = ramp_in + hold + ramp_out;
        if self.elapsed_secs >= total {
            return None;
        }

        if ramp_in > 0.0 && self.elapsed_secs < ramp_in {
            let progress = self.elapsed_secs / ramp_in;
            let eased = self.easing.sample_clamped(progress);
            return Some(1.0 + (self.scale - 1.0) * eased);
        }

        if self.elapsed_secs < ramp_in + hold {
            return Some(self.scale);
        }

        if ramp_out <= 0.0 {
            return None;
        }

        let progress = (self.elapsed_secs - ramp_in - hold) / ramp_out;
        let eased = self.easing.sample_clamped(progress.clamp(0.0, 1.0));
        Some(self.scale + (1.0 - self.scale) * eased)
    }
}

#[must_use]
pub fn resolve_effective_time_scale(
    global: &GlobalTimeScale,
    local: Option<&EntityTimeScale>,
    ignore_global: bool,
    ignore_hitstop: bool,
) -> f32 {
    let global_scale = if ignore_global {
        1.0
    } else if ignore_hitstop {
        global.base_scale
    } else {
        global.scale
    };

    let local_scale = local.map_or(1.0, |local_scale| {
        if ignore_hitstop {
            local_scale.base_scale
        } else {
            local_scale.scale
        }
    });

    global_scale * local_scale
}

pub(crate) fn advance_global_clock(
    time: Res<Time<Real>>,
    mut global_time: ResMut<GlobalTimeScale>,
) {
    global_time.frame_index = global_time.frame_index.saturating_add(1);
    global_time.unscaled_delta_secs = time.delta_secs();
    global_time.elapsed_unscaled_secs += global_time.unscaled_delta_secs;
}

pub(crate) fn resolve_global_time_scale(
    mut global_time: ResMut<GlobalTimeScale>,
    mut runtime: ResMut<WorldTimeScaleRuntime>,
    mut diagnostics: ResMut<GameFeelDiagnostics>,
) {
    let delta = global_time.unscaled_delta_secs;
    runtime.ramps.retain_mut(|effect| {
        effect.elapsed_secs += delta;
        effect.sample().is_some()
    });

    global_time.base_scale = resolve_ramp_scale(&runtime.ramps);
    global_time.hitstop_scale = runtime
        .hitstop
        .and_then(sample_hitstop)
        .unwrap_or(1.0)
        .clamp(0.0, 1.0);
    if let Some(hitstop) = runtime.hitstop.as_mut() {
        advance_hitstop(hitstop);
        let clear_hitstop = (hitstop.hold_frames_remaining == 0
            && hitstop.recovery_frames_total > 0
            && hitstop.recovery_frames_elapsed >= hitstop.recovery_frames_total)
            || (hitstop.hold_frames_remaining == 0
                && hitstop.recovery_frames_total == 0
                && hitstop.recovery_frames_elapsed > 0);
        if clear_hitstop {
            runtime.hitstop = None;
        }
    }

    global_time.scale = (global_time.base_scale * global_time.hitstop_scale).clamp(0.0, 4.0);
    global_time.scaled_delta_secs = global_time.unscaled_delta_secs * global_time.scale;
    global_time.elapsed_scaled_secs += global_time.scaled_delta_secs;

    diagnostics.frame_index = global_time.frame_index;
    diagnostics.global_base_scale = global_time.base_scale;
    diagnostics.global_hitstop_scale = global_time.hitstop_scale;
    diagnostics.global_scale = global_time.scale;
}

pub(crate) fn update_local_time_scales(
    global: Res<GlobalTimeScale>,
    mut diagnostics: ResMut<GameFeelDiagnostics>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut LocalTimeScaleRuntime,
        Option<&mut EntityTimeScale>,
    )>,
) {
    diagnostics.active_local_time_targets = 0;

    for (entity, mut runtime, output) in &mut query {
        runtime.ramps.retain_mut(|effect| {
            effect.elapsed_secs += global.unscaled_delta_secs;
            effect.sample().is_some()
        });
        let base_scale = resolve_ramp_scale(&runtime.ramps);
        let hitstop_scale = runtime
            .hitstop
            .and_then(sample_hitstop)
            .unwrap_or(1.0)
            .clamp(0.0, 1.0);
        if let Some(hitstop) = runtime.hitstop.as_mut() {
            advance_hitstop(hitstop);
            let clear_hitstop = (hitstop.hold_frames_remaining == 0
                && hitstop.recovery_frames_total > 0
                && hitstop.recovery_frames_elapsed >= hitstop.recovery_frames_total)
                || (hitstop.hold_frames_remaining == 0
                    && hitstop.recovery_frames_total == 0
                    && hitstop.recovery_frames_elapsed > 0);
            if clear_hitstop {
                runtime.hitstop = None;
            }
        }

        let composed = EntityTimeScale {
            base_scale,
            hitstop_scale,
            scale: (base_scale * hitstop_scale).clamp(0.0, 4.0),
        };

        if runtime.ramps.is_empty() && runtime.hitstop.is_none() {
            if output.is_some() {
                commands.entity(entity).remove::<EntityTimeScale>();
            }
            commands.entity(entity).remove::<LocalTimeScaleRuntime>();
            continue;
        }

        diagnostics.active_local_time_targets += 1;
        match output {
            Some(mut output) => {
                *output = composed;
            }
            None => {
                commands.entity(entity).insert(composed);
            }
        }
    }
}

pub(crate) fn resolve_ramp_scale(ramps: &[TimeScaleRamp]) -> f32 {
    let mut best_priority = i32::MIN;
    let mut best_scale = 1.0;

    for ramp in ramps {
        let Some(scale) = ramp.sample() else {
            continue;
        };
        if ramp.priority > best_priority {
            best_priority = ramp.priority;
            best_scale = scale;
        } else if ramp.priority == best_priority {
            best_scale = best_scale.min(scale);
        }
    }

    best_scale.clamp(0.0, 4.0)
}

pub(crate) fn apply_hitstop_stacking(
    slot: &mut Option<ActiveHitstop>,
    hold_frames: u32,
    recovery_frames: u32,
    stacking: HitstopStacking,
) {
    match slot {
        None => {
            *slot = Some(ActiveHitstop {
                hold_frames_remaining: hold_frames,
                recovery_frames_total: recovery_frames,
                recovery_frames_elapsed: 0,
            });
        }
        Some(existing) => match stacking {
            HitstopStacking::Refresh => {
                *existing = ActiveHitstop {
                    hold_frames_remaining: hold_frames,
                    recovery_frames_total: recovery_frames,
                    recovery_frames_elapsed: 0,
                };
            }
            HitstopStacking::Max => {
                existing.hold_frames_remaining = existing.hold_frames_remaining.max(hold_frames);
                existing.recovery_frames_total =
                    existing.recovery_frames_total.max(recovery_frames);
                existing.recovery_frames_elapsed = 0;
            }
            HitstopStacking::AdditiveWithCap { frame_cap } => {
                existing.hold_frames_remaining = existing
                    .hold_frames_remaining
                    .saturating_add(hold_frames)
                    .min(frame_cap);
                existing.recovery_frames_total =
                    existing.recovery_frames_total.max(recovery_frames);
                existing.recovery_frames_elapsed = 0;
            }
            HitstopStacking::IgnoreWeaker => {
                let existing_strength =
                    existing.hold_frames_remaining + existing.recovery_frames_total;
                let incoming_strength = hold_frames + recovery_frames;
                if incoming_strength >= existing_strength {
                    *existing = ActiveHitstop {
                        hold_frames_remaining: hold_frames,
                        recovery_frames_total: recovery_frames,
                        recovery_frames_elapsed: 0,
                    };
                }
            }
        },
    }
}

fn sample_hitstop(hitstop: ActiveHitstop) -> Option<f32> {
    if hitstop.hold_frames_remaining > 0 {
        return Some(0.0);
    }
    if hitstop.recovery_frames_total == 0 {
        return None;
    }
    let progress =
        hitstop.recovery_frames_elapsed as f32 / hitstop.recovery_frames_total.max(1) as f32;
    Some(progress.clamp(0.0, 1.0))
}

fn advance_hitstop(hitstop: &mut ActiveHitstop) {
    if hitstop.hold_frames_remaining > 0 {
        hitstop.hold_frames_remaining -= 1;
        return;
    }
    if hitstop.recovery_frames_total > 0 {
        hitstop.recovery_frames_elapsed = hitstop.recovery_frames_elapsed.saturating_add(1);
    }
}

#[cfg(test)]
#[path = "time_scale_tests.rs"]
mod tests;
