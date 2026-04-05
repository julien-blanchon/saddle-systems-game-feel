use bevy::prelude::*;

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EffectTimeDomain {
    #[default]
    Unscaled,
    GlobalScaled,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq)]
pub struct DistanceAttenuation {
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub exponent: f32,
}

impl DistanceAttenuation {
    #[must_use]
    pub fn sample(self, distance: f32) -> f32 {
        if distance <= self.inner_radius {
            return 1.0;
        }
        if distance >= self.outer_radius {
            return 0.0;
        }

        let span = (self.outer_radius - self.inner_radius).max(f32::EPSILON);
        let normalized = 1.0 - ((distance - self.inner_radius) / span);
        normalized.clamp(0.0, 1.0).powf(self.exponent.max(0.01))
    }
}

impl Default for DistanceAttenuation {
    fn default() -> Self {
        Self {
            inner_radius: 0.0,
            outer_radius: 10.0,
            exponent: 1.0,
        }
    }
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ScreenPulsePresentation {
    #[default]
    OutputOnly,
    LegacyBuiltIn,
}

#[derive(Resource, Reflect, Clone, Debug)]
#[reflect(Resource, Default)]
pub struct GameFeelConfig {
    pub screen_presentation: ScreenPulsePresentation,
    pub overlay_flash_color: Color,
    pub overlay_vignette_color: Color,
    pub overlay_border_fraction: f32,
    pub overlay_z_index: i32,
}

impl Default for GameFeelConfig {
    fn default() -> Self {
        Self {
            screen_presentation: ScreenPulsePresentation::OutputOnly,
            overlay_flash_color: Color::WHITE,
            overlay_vignette_color: Color::BLACK,
            overlay_border_fraction: 0.18,
            overlay_z_index: 1_000,
        }
    }
}

#[derive(Resource, Reflect, Clone, Debug)]
#[reflect(Resource, Default)]
pub struct GameFeelToggles {
    pub shake_enabled: bool,
    pub punch_enabled: bool,
    pub flash_enabled: bool,
    pub hitstop_enabled: bool,
    pub time_scale_enabled: bool,
    pub rumble_enabled: bool,
    pub squash_stretch_enabled: bool,
    pub knockback_enabled: bool,
    pub screen_pulse_enabled: bool,
}

impl Default for GameFeelToggles {
    fn default() -> Self {
        Self {
            shake_enabled: true,
            punch_enabled: true,
            flash_enabled: true,
            hitstop_enabled: true,
            time_scale_enabled: true,
            rumble_enabled: true,
            squash_stretch_enabled: true,
            knockback_enabled: true,
            screen_pulse_enabled: true,
        }
    }
}

impl GameFeelToggles {
    #[must_use]
    pub fn all_enabled() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn all_disabled() -> Self {
        Self {
            shake_enabled: false,
            punch_enabled: false,
            flash_enabled: false,
            hitstop_enabled: false,
            time_scale_enabled: false,
            rumble_enabled: false,
            squash_stretch_enabled: false,
            knockback_enabled: false,
            screen_pulse_enabled: false,
        }
    }
}

#[derive(Resource, Reflect, Clone, Debug, Default)]
#[reflect(Resource, Default)]
pub struct GameFeelDiagnostics {
    pub frame_index: u64,
    pub active_shake_listeners: usize,
    pub active_punch_listeners: usize,
    pub active_screen_pulses: usize,
    pub active_rumble_listeners: usize,
    pub active_entity_flashes: usize,
    pub active_scale_effects: usize,
    pub active_knockback_effects: usize,
    pub active_recipe_players: usize,
    pub active_local_time_targets: usize,
    pub global_base_scale: f32,
    pub global_hitstop_scale: f32,
    pub global_scale: f32,
}
