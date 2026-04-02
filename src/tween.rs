use bevy::{
    math::curve::{Curve, easing::EaseFunction},
    prelude::*,
};

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TweenRepeat {
    #[default]
    Once,
    Repeat {
        loops: u32,
    },
    PingPong {
        loops: u32,
    },
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq)]
pub struct Tween {
    pub delay_secs: f32,
    pub duration_secs: f32,
    pub easing: EaseFunction,
    pub repeat: TweenRepeat,
}

impl Default for Tween {
    fn default() -> Self {
        Self {
            delay_secs: 0.0,
            duration_secs: 0.2,
            easing: EaseFunction::SineInOut,
            repeat: TweenRepeat::Once,
        }
    }
}

impl Tween {
    #[must_use]
    pub fn sample(self, elapsed_secs: f32) -> TweenSample {
        if elapsed_secs < self.delay_secs {
            return TweenSample {
                active: false,
                finished: false,
                progress: 0.0,
                eased: 0.0,
            };
        }

        let duration = self.duration_secs.max(f32::EPSILON);
        let local = elapsed_secs - self.delay_secs;
        let (progress, finished) = match self.repeat {
            TweenRepeat::Once => {
                let progress = (local / duration).clamp(0.0, 1.0);
                (progress, local >= duration)
            }
            TweenRepeat::Repeat { loops } => {
                let total_duration = duration * (loops.saturating_add(1) as f32);
                let finished = local >= total_duration;
                let clamped_local = if finished {
                    total_duration
                } else {
                    local.max(0.0)
                };
                let cycle = (clamped_local / duration).floor();
                let cycle_progress = if finished {
                    1.0
                } else {
                    (clamped_local / duration) - cycle
                };
                (cycle_progress.clamp(0.0, 1.0), finished)
            }
            TweenRepeat::PingPong { loops } => {
                let total_legs = loops.saturating_add(1) as f32;
                let total_duration = duration * total_legs;
                let finished = local >= total_duration;
                let clamped_local = if finished {
                    total_duration
                } else {
                    local.max(0.0)
                };
                let leg = (clamped_local / duration).floor() as u32;
                let mut leg_progress = if finished {
                    1.0
                } else {
                    (clamped_local / duration) - leg as f32
                };
                if leg % 2 == 1 {
                    leg_progress = 1.0 - leg_progress;
                }
                (leg_progress.clamp(0.0, 1.0), finished)
            }
        };

        TweenSample {
            active: true,
            finished,
            progress,
            eased: self.easing.sample_clamped(progress),
        }
    }

    #[must_use]
    pub fn total_duration_secs(self) -> f32 {
        let legs = match self.repeat {
            TweenRepeat::Once => 1,
            TweenRepeat::Repeat { loops } | TweenRepeat::PingPong { loops } => {
                loops.saturating_add(1)
            }
        };
        self.delay_secs + self.duration_secs.max(f32::EPSILON) * legs as f32
    }
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq)]
pub struct TweenSample {
    pub active: bool,
    pub finished: bool,
    pub progress: f32,
    pub eased: f32,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq)]
pub struct AttackSustainDecay {
    pub attack_secs: f32,
    pub sustain_secs: f32,
    pub decay_secs: f32,
    pub attack_easing: EaseFunction,
    pub decay_easing: EaseFunction,
}

impl Default for AttackSustainDecay {
    fn default() -> Self {
        Self {
            attack_secs: 0.05,
            sustain_secs: 0.0,
            decay_secs: 0.15,
            attack_easing: EaseFunction::SineOut,
            decay_easing: EaseFunction::SineIn,
        }
    }
}

impl AttackSustainDecay {
    #[must_use]
    pub fn sample(self, elapsed_secs: f32) -> f32 {
        if elapsed_secs <= 0.0 {
            return 0.0;
        }

        if elapsed_secs < self.attack_secs {
            let progress = elapsed_secs / self.attack_secs.max(f32::EPSILON);
            return self.attack_easing.sample_clamped(progress);
        }

        let sustain_end = self.attack_secs + self.sustain_secs.max(0.0);
        if elapsed_secs < sustain_end {
            return 1.0;
        }

        let decay_elapsed = elapsed_secs - sustain_end;
        if decay_elapsed < self.decay_secs {
            let progress = decay_elapsed / self.decay_secs.max(f32::EPSILON);
            return 1.0 - self.decay_easing.sample_clamped(progress);
        }

        0.0
    }

    #[must_use]
    pub fn total_duration_secs(self) -> f32 {
        self.attack_secs + self.sustain_secs + self.decay_secs
    }
}

#[cfg(test)]
#[path = "tween_tests.rs"]
mod tests;
