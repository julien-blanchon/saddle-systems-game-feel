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
}
