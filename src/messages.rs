use crate::{
    channels::GameFeelChannels,
    config::{DistanceAttenuation, EffectTimeDomain},
    flash::FlashTarget,
    punch::{ImpulseSpace, PunchProfile},
    recipe::FeedbackContext,
    shake::ShakeProfile,
    squash::{ScaleEffectMode, ScaleStacking},
    time_scale::{HitstopStacking, TimeScaleTarget},
};
use bevy::{math::curve::easing::EaseFunction, prelude::*};

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ListenerTarget {
    #[default]
    All,
    Channels(GameFeelChannels),
    Entity(Entity),
}

#[derive(Message, Reflect, Clone, Debug, PartialEq)]
pub struct AddTrauma {
    pub target: ListenerTarget,
    pub trauma: f32,
    pub origin: Option<Vec3>,
    pub attenuation: Option<DistanceAttenuation>,
    pub propagation_speed: Option<f32>,
    pub directional_bias: Vec3,
    pub profile_override: Option<ShakeProfile>,
}

impl AddTrauma {
    #[must_use]
    pub fn new(target: ListenerTarget, trauma: f32) -> Self {
        Self {
            target,
            trauma,
            origin: None,
            attenuation: None,
            propagation_speed: None,
            directional_bias: Vec3::ZERO,
            profile_override: None,
        }
    }

    #[must_use]
    pub fn with_origin(mut self, origin: Vec3) -> Self {
        self.origin = Some(origin);
        self
    }

    #[must_use]
    pub fn with_attenuation(mut self, attenuation: DistanceAttenuation) -> Self {
        self.attenuation = Some(attenuation);
        self
    }

    #[must_use]
    pub fn with_propagation_speed(mut self, speed: f32) -> Self {
        self.propagation_speed = Some(speed);
        self
    }

    #[must_use]
    pub fn with_directional_bias(mut self, bias: Vec3) -> Self {
        self.directional_bias = bias;
        self
    }

    #[must_use]
    pub fn with_profile(mut self, profile: ShakeProfile) -> Self {
        self.profile_override = Some(profile);
        self
    }
}

#[derive(Message, Reflect, Clone, Debug, PartialEq)]
pub struct RequestCameraImpulse {
    pub target: ListenerTarget,
    pub translation: Vec3,
    pub rotation: Vec3,
    pub fov: f32,
    pub origin: Option<Vec3>,
    pub attenuation: Option<DistanceAttenuation>,
    pub propagation_speed: Option<f32>,
    pub space: ImpulseSpace,
    pub profile_override: Option<PunchProfile>,
}

impl RequestCameraImpulse {
    #[must_use]
    pub fn new(target: ListenerTarget, translation: Vec3) -> Self {
        Self {
            target,
            translation,
            rotation: Vec3::ZERO,
            fov: 0.0,
            origin: None,
            attenuation: None,
            propagation_speed: None,
            space: ImpulseSpace::Local,
            profile_override: None,
        }
    }

    #[must_use]
    pub fn with_rotation(mut self, rotation: Vec3) -> Self {
        self.rotation = rotation;
        self
    }

    #[must_use]
    pub fn with_fov(mut self, fov: f32) -> Self {
        self.fov = fov;
        self
    }

    #[must_use]
    pub fn with_origin(mut self, origin: Vec3) -> Self {
        self.origin = Some(origin);
        self
    }

    #[must_use]
    pub fn with_attenuation(mut self, attenuation: DistanceAttenuation) -> Self {
        self.attenuation = Some(attenuation);
        self
    }

    #[must_use]
    pub fn with_propagation_speed(mut self, speed: f32) -> Self {
        self.propagation_speed = Some(speed);
        self
    }

    #[must_use]
    pub fn in_world_space(mut self) -> Self {
        self.space = ImpulseSpace::World;
        self
    }

    #[must_use]
    pub fn with_profile(mut self, profile: PunchProfile) -> Self {
        self.profile_override = Some(profile);
        self
    }
}

#[derive(Message, Reflect, Clone, Debug, PartialEq)]
pub struct RequestTimeScale {
    pub target: TimeScaleTarget,
    pub scale: f32,
    pub ramp_in_secs: f32,
    pub hold_secs: f32,
    pub ramp_out_secs: f32,
    pub easing: EaseFunction,
    pub priority: i32,
    pub group_entities: Vec<Entity>,
}

impl RequestTimeScale {
    #[must_use]
    pub fn new(target: TimeScaleTarget, scale: f32) -> Self {
        Self {
            target,
            scale,
            ramp_in_secs: 0.0,
            hold_secs: 0.0,
            ramp_out_secs: 0.0,
            easing: EaseFunction::SineInOut,
            priority: 0,
            group_entities: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_ramp(mut self, ramp_in: f32, hold: f32, ramp_out: f32) -> Self {
        self.ramp_in_secs = ramp_in;
        self.hold_secs = hold;
        self.ramp_out_secs = ramp_out;
        self
    }

    #[must_use]
    pub fn with_easing(mut self, easing: EaseFunction) -> Self {
        self.easing = easing;
        self
    }

    #[must_use]
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    #[must_use]
    pub fn with_group(mut self, entities: Vec<Entity>) -> Self {
        self.group_entities = entities;
        self
    }
}

#[derive(Message, Reflect, Clone, Debug, PartialEq)]
pub struct RequestHitstop {
    pub target: TimeScaleTarget,
    pub hold_frames: u32,
    pub recovery_frames: u32,
    pub stacking: HitstopStacking,
    pub group_entities: Vec<Entity>,
}

impl RequestHitstop {
    #[must_use]
    pub fn new(target: TimeScaleTarget, hold_frames: u32) -> Self {
        Self {
            target,
            hold_frames,
            recovery_frames: 0,
            stacking: HitstopStacking::Refresh,
            group_entities: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_recovery(mut self, frames: u32) -> Self {
        self.recovery_frames = frames;
        self
    }

    #[must_use]
    pub fn with_stacking(mut self, stacking: HitstopStacking) -> Self {
        self.stacking = stacking;
        self
    }

    #[must_use]
    pub fn with_group(mut self, entities: Vec<Entity>) -> Self {
        self.group_entities = entities;
        self
    }
}

#[derive(Message, Reflect, Clone, Debug, PartialEq)]
pub struct RequestSplitHitstop {
    pub attacker: Entity,
    pub attacker_frames: u32,
    pub target: Entity,
    pub target_frames: u32,
    pub recovery_frames: u32,
    pub stacking: HitstopStacking,
}

#[derive(Message, Reflect, Clone, Debug, PartialEq)]
pub struct RequestFlash {
    pub target: FlashTarget,
    pub color: Color,
    pub intensity: f32,
    pub chromatic_aberration: f32,
    pub vignette: f32,
    pub origin: Option<Vec3>,
    pub attenuation: Option<DistanceAttenuation>,
    pub duration_secs: f32,
    pub easing: EaseFunction,
    pub clock: EffectTimeDomain,
}

impl RequestFlash {
    #[must_use]
    pub fn new(target: FlashTarget, color: Color, intensity: f32) -> Self {
        Self {
            target,
            color,
            intensity,
            chromatic_aberration: 0.0,
            vignette: 0.0,
            origin: None,
            attenuation: None,
            duration_secs: 0.12,
            easing: EaseFunction::SineOut,
            clock: EffectTimeDomain::Unscaled,
        }
    }

    #[must_use]
    pub fn with_chromatic_aberration(mut self, amount: f32) -> Self {
        self.chromatic_aberration = amount;
        self
    }

    #[must_use]
    pub fn with_vignette(mut self, amount: f32) -> Self {
        self.vignette = amount;
        self
    }

    #[must_use]
    pub fn with_origin(mut self, origin: Vec3) -> Self {
        self.origin = Some(origin);
        self
    }

    #[must_use]
    pub fn with_attenuation(mut self, attenuation: DistanceAttenuation) -> Self {
        self.attenuation = Some(attenuation);
        self
    }

    #[must_use]
    pub fn with_duration(mut self, secs: f32) -> Self {
        self.duration_secs = secs;
        self
    }

    #[must_use]
    pub fn with_easing(mut self, easing: EaseFunction) -> Self {
        self.easing = easing;
        self
    }

    #[must_use]
    pub fn scaled_by_time(mut self) -> Self {
        self.clock = EffectTimeDomain::GlobalScaled;
        self
    }
}

#[derive(Message, Reflect, Clone, Debug, PartialEq)]
pub struct RequestRumble {
    pub target: ListenerTarget,
    pub low_frequency: f32,
    pub high_frequency: f32,
    pub duration_secs: f32,
    pub easing: EaseFunction,
    pub clock: EffectTimeDomain,
}

impl RequestRumble {
    #[must_use]
    pub fn new(target: ListenerTarget, low_frequency: f32, high_frequency: f32) -> Self {
        Self {
            target,
            low_frequency,
            high_frequency,
            duration_secs: 0.18,
            easing: EaseFunction::SineOut,
            clock: EffectTimeDomain::Unscaled,
        }
    }

    #[must_use]
    pub fn with_duration(mut self, secs: f32) -> Self {
        self.duration_secs = secs;
        self
    }

    #[must_use]
    pub fn with_easing(mut self, easing: EaseFunction) -> Self {
        self.easing = easing;
        self
    }

    #[must_use]
    pub fn scaled_by_time(mut self) -> Self {
        self.clock = EffectTimeDomain::GlobalScaled;
        self
    }
}

#[derive(Message, Reflect, Clone, Debug, PartialEq)]
pub struct RequestSquashStretch {
    pub target: Entity,
    pub peak_scale: Vec3,
    pub mode: ScaleEffectMode,
    pub duration_secs: f32,
    pub easing: EaseFunction,
    pub clock: EffectTimeDomain,
    pub stacking: ScaleStacking,
    pub direction: Option<Vec3>,
    pub directional_magnitude: f32,
}

impl RequestSquashStretch {
    #[must_use]
    pub fn new(target: Entity, peak_scale: Vec3) -> Self {
        Self {
            target,
            peak_scale,
            mode: ScaleEffectMode::Relative,
            duration_secs: 0.18,
            easing: EaseFunction::BackOut,
            clock: EffectTimeDomain::Unscaled,
            stacking: ScaleStacking::Multiply,
            direction: None,
            directional_magnitude: 0.0,
        }
    }

    #[must_use]
    pub fn with_mode(mut self, mode: ScaleEffectMode) -> Self {
        self.mode = mode;
        self
    }

    #[must_use]
    pub fn with_duration(mut self, secs: f32) -> Self {
        self.duration_secs = secs;
        self
    }

    #[must_use]
    pub fn with_easing(mut self, easing: EaseFunction) -> Self {
        self.easing = easing;
        self
    }

    #[must_use]
    pub fn with_stacking(mut self, stacking: ScaleStacking) -> Self {
        self.stacking = stacking;
        self
    }

    #[must_use]
    pub fn with_direction(mut self, direction: Vec3, magnitude: f32) -> Self {
        self.direction = Some(direction);
        self.directional_magnitude = magnitude;
        self
    }

    #[must_use]
    pub fn scaled_by_time(mut self) -> Self {
        self.clock = EffectTimeDomain::GlobalScaled;
        self
    }
}

#[derive(Message, Reflect, Clone, Debug, PartialEq)]
pub struct PlayFeedbackRecipe {
    pub name: String,
    pub context: FeedbackContext,
}

impl PlayFeedbackRecipe {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            context: FeedbackContext::default(),
        }
    }

    #[must_use]
    pub fn with_context(mut self, context: FeedbackContext) -> Self {
        self.context = context;
        self
    }

    #[must_use]
    pub fn with_listener(mut self, listener: Entity) -> Self {
        self.context.listener = Some(listener);
        self
    }

    #[must_use]
    pub fn with_target(mut self, target: Entity) -> Self {
        self.context.target = Some(target);
        self
    }

    #[must_use]
    pub fn with_origin(mut self, origin: Vec3) -> Self {
        self.context.origin = Some(origin);
        self
    }

    #[must_use]
    pub fn with_direction(mut self, direction: Vec3) -> Self {
        self.context.direction = direction;
        self
    }

    #[must_use]
    pub fn with_channels(mut self, channels: GameFeelChannels) -> Self {
        self.context.channels = channels;
        self
    }

    #[must_use]
    pub fn with_intensity(mut self, multiplier: f32) -> Self {
        self.context.intensity_multiplier = multiplier;
        self
    }
}

#[derive(Message, Reflect, Clone, Debug, PartialEq)]
pub struct RequestKnockback {
    pub target: Entity,
    pub direction: Vec3,
    pub force: f32,
    pub duration_secs: f32,
    pub easing: EaseFunction,
    pub clock: EffectTimeDomain,
}

impl RequestKnockback {
    #[must_use]
    pub fn new(target: Entity, direction: Vec3, force: f32) -> Self {
        Self {
            target,
            direction,
            force,
            duration_secs: 0.25,
            easing: EaseFunction::ExponentialOut,
            clock: EffectTimeDomain::Unscaled,
        }
    }

    #[must_use]
    pub fn with_duration(mut self, secs: f32) -> Self {
        self.duration_secs = secs;
        self
    }

    #[must_use]
    pub fn with_easing(mut self, easing: EaseFunction) -> Self {
        self.easing = easing;
        self
    }

    #[must_use]
    pub fn scaled_by_time(mut self) -> Self {
        self.clock = EffectTimeDomain::GlobalScaled;
        self
    }
}
