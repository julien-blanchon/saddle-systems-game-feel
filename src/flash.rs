use crate::{
    channels::GameFeelChannels,
    config::{EffectTimeDomain, GameFeelConfig, GameFeelDiagnostics},
    time_scale::GlobalTimeScale,
    tween::Tween,
};
use bevy::{post_process::effect_stack::ChromaticAberration, prelude::*};

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct ScreenPulseListener {
    pub channels: GameFeelChannels,
    pub flash_multiplier: f32,
    pub chromatic_multiplier: f32,
    pub vignette_multiplier: f32,
}

impl Default for ScreenPulseListener {
    fn default() -> Self {
        Self {
            channels: GameFeelChannels::default(),
            flash_multiplier: 1.0,
            chromatic_multiplier: 1.0,
            vignette_multiplier: 1.0,
        }
    }
}

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct FlashOutput {
    pub color: Color,
    pub intensity: f32,
}

impl Default for FlashOutput {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 0.0,
        }
    }
}

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct ScreenPulseOutput {
    pub color: Color,
    pub flash_alpha: f32,
    pub chromatic_aberration: f32,
    pub vignette: f32,
}

impl Default for ScreenPulseOutput {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            flash_alpha: 0.0,
            chromatic_aberration: 0.0,
            vignette: 0.0,
        }
    }
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq)]
pub enum FlashTarget {
    Entity(Entity),
    Screen(crate::messages::ListenerTarget),
    EntityAndScreen {
        entity: Entity,
        screen: crate::messages::ListenerTarget,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct ActiveEntityFlash {
    pub color: Color,
    pub intensity: f32,
    pub tween: Tween,
    pub clock: EffectTimeDomain,
    pub elapsed_secs: f32,
}

#[derive(Component, Debug, Default)]
pub(crate) struct EntityFlashRuntime {
    pub effects: Vec<ActiveEntityFlash>,
}

#[derive(Clone, Debug)]
pub(crate) struct ActiveScreenPulse {
    pub color: Color,
    pub flash_alpha: f32,
    pub chromatic_aberration: f32,
    pub vignette: f32,
    pub tween: Tween,
    pub clock: EffectTimeDomain,
    pub elapsed_secs: f32,
}

#[derive(Component, Debug, Default)]
pub(crate) struct ScreenPulseRuntime {
    pub effects: Vec<ActiveScreenPulse>,
}

#[derive(Component, Debug, Default)]
pub(crate) struct PresentedSpriteState {
    pub original_color: Color,
    pub active_last_frame: bool,
}

#[derive(Component, Debug, Default)]
pub(crate) struct PresentedChromaticState {
    pub original_intensity: f32,
    pub original_max_samples: u32,
    pub active_last_frame: bool,
}

#[derive(Component, Debug)]
pub(crate) struct ScreenOverlayNodes {
    pub flash_fill: Entity,
    pub top: Entity,
    pub bottom: Entity,
    pub left: Entity,
    pub right: Entity,
}

#[derive(Component, Debug)]
pub(crate) struct ScreenOverlayRoot;

#[derive(Component, Debug)]
pub(crate) struct ScreenOverlayOwner(pub Entity);

pub(crate) fn update_entity_flash_outputs(
    global: Res<GlobalTimeScale>,
    mut diagnostics: ResMut<GameFeelDiagnostics>,
    mut query: Query<(&mut FlashOutput, &mut EntityFlashRuntime)>,
) {
    diagnostics.active_entity_flashes = 0;

    for (mut output, mut runtime) in &mut query {
        runtime.effects.retain_mut(|effect| {
            effect.elapsed_secs += match effect.clock {
                EffectTimeDomain::Unscaled => global.unscaled_delta_secs,
                EffectTimeDomain::GlobalScaled => global.scaled_delta_secs,
            };
            !effect.tween.sample(effect.elapsed_secs).finished
        });

        if runtime.effects.is_empty() {
            *output = FlashOutput::default();
            continue;
        }

        diagnostics.active_entity_flashes += runtime.effects.len();

        let mut strongest = FlashOutput::default();
        for effect in &runtime.effects {
            let weight = effect.tween.sample(effect.elapsed_secs).eased * effect.intensity;
            if weight >= strongest.intensity {
                strongest = FlashOutput {
                    color: effect.color,
                    intensity: weight,
                };
            }
        }
        *output = strongest;
    }
}

pub(crate) fn update_screen_pulse_outputs(
    global: Res<GlobalTimeScale>,
    mut diagnostics: ResMut<GameFeelDiagnostics>,
    mut query: Query<(
        &ScreenPulseListener,
        &mut ScreenPulseOutput,
        &mut ScreenPulseRuntime,
    )>,
) {
    diagnostics.active_screen_pulses = 0;

    for (listener, mut output, mut runtime) in &mut query {
        runtime.effects.retain_mut(|effect| {
            effect.elapsed_secs += match effect.clock {
                EffectTimeDomain::Unscaled => global.unscaled_delta_secs,
                EffectTimeDomain::GlobalScaled => global.scaled_delta_secs,
            };
            !effect.tween.sample(effect.elapsed_secs).finished
        });

        if runtime.effects.is_empty() {
            *output = ScreenPulseOutput::default();
            continue;
        }

        diagnostics.active_screen_pulses += runtime.effects.len();

        let mut flash_alpha = 0.0_f32;
        let mut chromatic_aberration = 0.0_f32;
        let mut vignette = 0.0_f32;
        let mut color = Color::WHITE;

        for effect in &runtime.effects {
            let weight = effect.tween.sample(effect.elapsed_secs).eased;
            let scaled_flash = effect.flash_alpha * weight * listener.flash_multiplier;
            if scaled_flash >= flash_alpha {
                flash_alpha = scaled_flash;
                color = effect.color;
            }
            chromatic_aberration = chromatic_aberration
                .max(effect.chromatic_aberration * weight * listener.chromatic_multiplier);
            vignette = vignette.max(effect.vignette * weight * listener.vignette_multiplier);
        }

        *output = ScreenPulseOutput {
            color,
            flash_alpha,
            chromatic_aberration,
            vignette,
        };
    }
}

pub(crate) fn ensure_screen_overlays(
    mut commands: Commands,
    config: Res<GameFeelConfig>,
    query: Query<(Entity, Option<&ScreenOverlayNodes>), With<ScreenPulseListener>>,
) {
    for (listener, nodes) in &query {
        if nodes.is_some() {
            continue;
        }

        let root = commands
            .spawn((
                Name::new("Game Feel Screen Overlay"),
                ScreenOverlayRoot,
                ScreenOverlayOwner(listener),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                UiTargetCamera(listener),
                GlobalZIndex(config.overlay_z_index),
            ))
            .id();

        let flash_fill = commands
            .spawn((
                Name::new("Game Feel Flash Fill"),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .id();

        let border = (config.overlay_border_fraction * 100.0).clamp(0.0, 50.0);
        let top = commands
            .spawn((
                Name::new("Game Feel Vignette Top"),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(border),
                    top: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .id();
        let bottom = commands
            .spawn((
                Name::new("Game Feel Vignette Bottom"),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(border),
                    bottom: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .id();
        let left = commands
            .spawn((
                Name::new("Game Feel Vignette Left"),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(border),
                    height: Val::Percent(100.0),
                    left: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .id();
        let right = commands
            .spawn((
                Name::new("Game Feel Vignette Right"),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(border),
                    height: Val::Percent(100.0),
                    right: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .id();

        commands
            .entity(root)
            .add_children(&[flash_fill, top, bottom, left, right]);
        commands.entity(listener).insert(ScreenOverlayNodes {
            flash_fill,
            top,
            bottom,
            left,
            right,
        });
    }
}

pub(crate) fn cleanup_orphaned_overlays(
    mut commands: Commands,
    listener_query: Query<(), With<ScreenPulseListener>>,
    root_query: Query<(Entity, &ScreenOverlayOwner), With<ScreenOverlayRoot>>,
) {
    for (root, owner) in &root_query {
        if listener_query.get(owner.0).is_err() {
            commands.entity(root).despawn();
        }
    }
}

pub(crate) fn restore_presented_sprites(
    mut query: Query<(&mut Sprite, &mut PresentedSpriteState)>,
) {
    for (mut sprite, mut state) in &mut query {
        if state.active_last_frame {
            sprite.color = state.original_color;
            state.active_last_frame = false;
        }
    }
}

pub(crate) fn apply_sprite_flash_outputs(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Sprite,
        Option<&mut PresentedSpriteState>,
        &FlashOutput,
    )>,
) {
    for (entity, mut sprite, state, output) in &mut query {
        if output.intensity <= 0.001 {
            continue;
        }

        let original = sprite.color;
        let color = mix_color(original, output.color, output.intensity);
        sprite.color = color;

        match state {
            Some(mut state) => {
                state.original_color = original;
                state.active_last_frame = true;
            }
            None => {
                commands.entity(entity).insert(PresentedSpriteState {
                    original_color: original,
                    active_last_frame: true,
                });
            }
        }
    }
}

pub(crate) fn restore_presented_chromatic(
    mut query: Query<(&mut ChromaticAberration, &mut PresentedChromaticState)>,
) {
    for (mut chromatic, mut state) in &mut query {
        if state.active_last_frame {
            chromatic.intensity = state.original_intensity;
            chromatic.max_samples = state.original_max_samples;
            state.active_last_frame = false;
        }
    }
}

pub(crate) fn apply_chromatic_outputs(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &ScreenPulseOutput,
            &mut ChromaticAberration,
            Option<&mut PresentedChromaticState>,
        ),
        With<ScreenPulseListener>,
    >,
) {
    for (entity, output, mut chromatic, state) in &mut query {
        if output.chromatic_aberration <= 0.001 {
            continue;
        }

        let original_intensity = chromatic.intensity;
        let original_max_samples = chromatic.max_samples;
        chromatic.intensity = output.chromatic_aberration;
        chromatic.max_samples = (((output.chromatic_aberration - 0.02) / 0.18) * 56.0 + 8.0)
            .clamp(8.0, 64.0)
            .round() as u32;

        match state {
            Some(mut state) => {
                state.original_intensity = original_intensity;
                state.original_max_samples = original_max_samples;
                state.active_last_frame = true;
            }
            None => {
                commands.entity(entity).insert(PresentedChromaticState {
                    original_intensity,
                    original_max_samples,
                    active_last_frame: true,
                });
            }
        }
    }
}

pub(crate) fn apply_screen_overlay_outputs(
    config: Res<GameFeelConfig>,
    listener_query: Query<(&ScreenPulseOutput, &ScreenOverlayNodes)>,
    mut colors: Query<&mut BackgroundColor>,
) {
    for (output, nodes) in &listener_query {
        let flash = output.color.to_srgba();
        if let Ok(mut color) = colors.get_mut(nodes.flash_fill) {
            color.0 = Color::srgba(
                flash.red,
                flash.green,
                flash.blue,
                output.flash_alpha.clamp(0.0, 1.0),
            );
        }

        let vignette = config.overlay_vignette_color.to_srgba();
        let vignette_color = Color::srgba(
            vignette.red,
            vignette.green,
            vignette.blue,
            output.vignette.clamp(0.0, 1.0),
        );
        for entity in [nodes.top, nodes.bottom, nodes.left, nodes.right] {
            if let Ok(mut color) = colors.get_mut(entity) {
                color.0 = vignette_color;
            }
        }
    }
}

fn mix_color(base: Color, flash: Color, amount: f32) -> Color {
    let base = base.to_srgba();
    let flash = flash.to_srgba();
    let amount = amount.clamp(0.0, 1.0);
    Color::srgba(
        base.red + (flash.red - base.red) * amount,
        base.green + (flash.green - base.green) * amount,
        base.blue + (flash.blue - base.blue) * amount,
        base.alpha + (flash.alpha - base.alpha) * amount,
    )
}

#[cfg(test)]
#[path = "flash_tests.rs"]
mod tests;
