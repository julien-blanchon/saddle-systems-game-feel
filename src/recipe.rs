use crate::{
    channels::GameFeelChannels,
    config::{DistanceAttenuation, EffectTimeDomain, GameFeelDiagnostics},
    flash::FlashTarget,
    messages::{
        AddTrauma, ListenerTarget, PlayFeedbackRecipe, RequestCameraImpulse, RequestFlash,
        RequestHitstop, RequestKnockback, RequestRumble, RequestSquashStretch, RequestTimeScale,
    },
    punch::ImpulseSpace,
    time_scale::{HitstopStacking, TimeScaleTarget},
};
use bevy::{math::curve::easing::EaseFunction, prelude::*};
use std::collections::HashMap;

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct FeedbackContext {
    pub listener: Option<Entity>,
    pub target: Option<Entity>,
    pub group: Vec<Entity>,
    pub origin: Option<Vec3>,
    pub direction: Vec3,
    pub channels: GameFeelChannels,
    pub intensity_multiplier: f32,
}

impl Default for FeedbackContext {
    fn default() -> Self {
        Self {
            listener: None,
            target: None,
            group: Vec::new(),
            origin: None,
            direction: Vec3::ZERO,
            channels: GameFeelChannels::default(),
            intensity_multiplier: 1.0,
        }
    }
}

#[derive(Reflect, Clone, Debug, PartialEq, Eq, Default)]
pub enum FeedbackCondition {
    #[default]
    Always,
    RequiresListener,
    RequiresTarget,
    RequiresOrigin,
    RequiresGroup,
    Channels(GameFeelChannels),
}

impl FeedbackCondition {
    fn matches(&self, context: &FeedbackContext) -> bool {
        match self {
            Self::Always => true,
            Self::RequiresListener => context.listener.is_some(),
            Self::RequiresTarget => context.target.is_some(),
            Self::RequiresOrigin => context.origin.is_some(),
            Self::RequiresGroup => !context.group.is_empty(),
            Self::Channels(channels) => context.channels.intersects(*channels),
        }
    }
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EntitySelector {
    #[default]
    ContextTarget,
    Specific(Entity),
}

impl EntitySelector {
    fn resolve(self, context: &FeedbackContext) -> Option<Entity> {
        match self {
            Self::ContextTarget => context.target,
            Self::Specific(entity) => Some(entity),
        }
    }
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ListenerSelector {
    #[default]
    ContextListener,
    Specific(Entity),
    Channels(GameFeelChannels),
    All,
}

impl ListenerSelector {
    fn resolve(self, context: &FeedbackContext) -> ListenerTarget {
        match self {
            Self::ContextListener => context.listener.map_or(
                ListenerTarget::Channels(context.channels),
                ListenerTarget::Entity,
            ),
            Self::Specific(entity) => ListenerTarget::Entity(entity),
            Self::Channels(channels) => ListenerTarget::Channels(channels),
            Self::All => ListenerTarget::All,
        }
    }
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TimeScaleSelector {
    World,
    #[default]
    ContextTarget,
    ContextGroup,
    Specific(Entity),
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub enum RecipeFlashTarget {
    Entity(EntitySelector),
    Screen(ListenerSelector),
    EntityAndScreen {
        entity: EntitySelector,
        screen: ListenerSelector,
    },
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct RecipeTrauma {
    pub target: ListenerSelector,
    pub trauma: f32,
    pub directional_bias: Vec3,
    pub use_context_origin: bool,
    pub attenuation: Option<DistanceAttenuation>,
    pub propagation_speed: Option<f32>,
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct RecipeImpulse {
    pub target: ListenerSelector,
    pub translation: Vec3,
    pub rotation: Vec3,
    pub fov: f32,
    pub use_context_origin: bool,
    pub attenuation: Option<DistanceAttenuation>,
    pub propagation_speed: Option<f32>,
    pub space: ImpulseSpace,
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct RecipeFlash {
    pub target: RecipeFlashTarget,
    pub color: Color,
    pub intensity: f32,
    pub chromatic_aberration: f32,
    pub vignette: f32,
    pub use_context_origin: bool,
    pub attenuation: Option<DistanceAttenuation>,
    pub duration_secs: f32,
    pub easing: EaseFunction,
    pub clock: EffectTimeDomain,
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct RecipeHitstop {
    pub target: TimeScaleSelector,
    pub hold_frames: u32,
    pub recovery_frames: u32,
    pub stacking: HitstopStacking,
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct RecipeTimeScale {
    pub target: TimeScaleSelector,
    pub scale: f32,
    pub ramp_in_secs: f32,
    pub hold_secs: f32,
    pub ramp_out_secs: f32,
    pub easing: EaseFunction,
    pub priority: i32,
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct RecipeRumble {
    pub target: ListenerSelector,
    pub low_frequency: f32,
    pub high_frequency: f32,
    pub duration_secs: f32,
    pub easing: EaseFunction,
    pub clock: EffectTimeDomain,
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct RecipeSquashStretch {
    pub target: EntitySelector,
    pub peak_scale: Vec3,
    pub mode: crate::squash::ScaleEffectMode,
    pub stacking: crate::squash::ScaleStacking,
    pub duration_secs: f32,
    pub easing: EaseFunction,
    pub clock: EffectTimeDomain,
    pub direction_from_context: bool,
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct RecipeKnockback {
    pub target: EntitySelector,
    pub direction_from_context: bool,
    pub force: f32,
    pub duration_secs: f32,
    pub easing: EaseFunction,
    pub clock: EffectTimeDomain,
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct RecipeHooks {
    pub audio_cue: Option<String>,
    pub particle_cue: Option<String>,
    pub target: Option<EntitySelector>,
    pub use_context_origin: bool,
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub enum FeedbackAction {
    Trauma(RecipeTrauma),
    CameraImpulse(RecipeImpulse),
    Flash(RecipeFlash),
    Hitstop(RecipeHitstop),
    TimeScale(RecipeTimeScale),
    Rumble(RecipeRumble),
    SquashStretch(RecipeSquashStretch),
    Knockback(RecipeKnockback),
    Hooks(RecipeHooks),
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct FeedbackStep {
    pub name: String,
    pub at_secs: f32,
    pub actions: Vec<FeedbackAction>,
}

#[derive(Reflect, Clone, Debug, PartialEq, Default)]
pub enum FeedbackRecipeRepeat {
    #[default]
    Once,
    Times {
        total_plays: u32,
        gap_secs: f32,
    },
}

#[derive(Reflect, Clone, Debug, PartialEq, Default)]
pub struct FeedbackRecipe {
    pub cooldown_secs: f32,
    pub condition: FeedbackCondition,
    pub repeat: FeedbackRecipeRepeat,
    pub steps: Vec<FeedbackStep>,
}

#[derive(Resource, Reflect, Clone, Debug)]
#[reflect(Resource, Default)]
pub struct FeedbackRecipeLibrary {
    pub recipes: HashMap<String, FeedbackRecipe>,
}

impl FeedbackRecipeLibrary {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            recipes: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: impl Into<String>, recipe: FeedbackRecipe) {
        self.recipes.insert(name.into(), recipe);
    }
}

impl Default for FeedbackRecipeLibrary {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Message, Reflect, Clone, Debug, PartialEq)]
pub struct FeedbackStepFired {
    pub recipe_name: String,
    pub step_name: String,
    pub step_index: usize,
    pub loop_index: u32,
}

#[derive(Message, Reflect, Clone, Debug, PartialEq)]
pub struct FeedbackHookTriggered {
    pub recipe_name: String,
    pub step_name: String,
    pub step_index: usize,
    pub loop_index: u32,
    pub audio_cue: Option<String>,
    pub particle_cue: Option<String>,
    pub target: Option<Entity>,
    pub origin: Option<Vec3>,
}

#[derive(Clone, Debug)]
pub(crate) struct ActiveRecipePlayer {
    pub recipe_name: String,
    pub elapsed_secs: f32,
    pub fired_steps: usize,
    pub completed_loops: u32,
    pub context: FeedbackContext,
}

#[derive(Resource, Debug, Default)]
pub(crate) struct RecipeRuntime {
    pub players: Vec<ActiveRecipePlayer>,
    pub cooldowns: HashMap<String, f32>,
}

pub(crate) fn update_recipe_cooldowns(
    global: Res<crate::time_scale::GlobalTimeScale>,
    mut runtime: ResMut<RecipeRuntime>,
) {
    runtime
        .cooldowns
        .retain(|_, ready_at| *ready_at > global.elapsed_unscaled_secs);
}

pub(crate) fn start_recipe_players(
    library: Res<FeedbackRecipeLibrary>,
    global: Res<crate::time_scale::GlobalTimeScale>,
    mut runtime: ResMut<RecipeRuntime>,
    mut requests: MessageReader<PlayFeedbackRecipe>,
) {
    for request in requests.read() {
        let Some(recipe) = library.recipes.get(&request.name) else {
            continue;
        };
        if !recipe.condition.matches(&request.context) {
            continue;
        }
        if runtime
            .cooldowns
            .get(&request.name)
            .is_some_and(|ready_at| *ready_at > global.elapsed_unscaled_secs)
        {
            continue;
        }

        runtime.players.push(ActiveRecipePlayer {
            recipe_name: request.name.clone(),
            elapsed_secs: 0.0,
            fired_steps: 0,
            completed_loops: 0,
            context: request.context.clone(),
        });
        if recipe.cooldown_secs > 0.0 {
            runtime.cooldowns.insert(
                request.name.clone(),
                global.elapsed_unscaled_secs + recipe.cooldown_secs,
            );
        }
    }
}

pub(crate) fn advance_recipe_players(
    global: Res<crate::time_scale::GlobalTimeScale>,
    library: Res<FeedbackRecipeLibrary>,
    mut diagnostics: ResMut<GameFeelDiagnostics>,
    mut runtime: ResMut<RecipeRuntime>,
    mut trauma_writer: MessageWriter<AddTrauma>,
    mut impulse_writer: MessageWriter<RequestCameraImpulse>,
    mut flash_writer: MessageWriter<RequestFlash>,
    mut hitstop_writer: MessageWriter<RequestHitstop>,
    mut rumble_writer: MessageWriter<RequestRumble>,
    mut time_scale_writer: MessageWriter<RequestTimeScale>,
    mut squash_writer: MessageWriter<RequestSquashStretch>,
    mut knockback_writer: MessageWriter<RequestKnockback>,
    mut step_writer: MessageWriter<FeedbackStepFired>,
    mut hook_writer: MessageWriter<FeedbackHookTriggered>,
) {
    diagnostics.active_recipe_players = runtime.players.len();

    runtime.players.retain_mut(|player| {
        let Some(recipe) = library.recipes.get(&player.recipe_name) else {
            return false;
        };
        player.elapsed_secs += global.unscaled_delta_secs;

        loop {
            while let Some(step) = recipe.steps.get(player.fired_steps) {
                if player.elapsed_secs < step.at_secs {
                    break;
                }

                let intensity = player.context.intensity_multiplier;

                for action in &step.actions {
                    match action {
                        FeedbackAction::Trauma(action) => {
                            trauma_writer.write(AddTrauma {
                                target: action.target.resolve(&player.context),
                                trauma: action.trauma * intensity,
                                origin: action
                                    .use_context_origin
                                    .then_some(player.context.origin)
                                    .flatten(),
                                attenuation: action.attenuation,
                                propagation_speed: action.propagation_speed,
                                directional_bias: if player.context.direction.length_squared()
                                    > f32::EPSILON
                                {
                                    player.context.direction.normalize_or_zero()
                                        * action.directional_bias.length()
                                        * intensity
                                } else {
                                    action.directional_bias * intensity
                                },
                                profile_override: None,
                            });
                        }
                        FeedbackAction::CameraImpulse(action) => {
                            impulse_writer.write(RequestCameraImpulse {
                                target: action.target.resolve(&player.context),
                                translation: action.translation * intensity,
                                rotation: action.rotation * intensity,
                                fov: action.fov * intensity,
                                origin: action
                                    .use_context_origin
                                    .then_some(player.context.origin)
                                    .flatten(),
                                attenuation: action.attenuation,
                                propagation_speed: action.propagation_speed,
                                space: action.space,
                                profile_override: None,
                            });
                        }
                        FeedbackAction::Flash(action) => {
                            let target = match &action.target {
                                RecipeFlashTarget::Entity(selector) => {
                                    selector.resolve(&player.context).map(FlashTarget::Entity)
                                }
                                RecipeFlashTarget::Screen(selector) => {
                                    Some(FlashTarget::Screen(selector.resolve(&player.context)))
                                }
                                RecipeFlashTarget::EntityAndScreen { entity, screen } => entity
                                    .resolve(&player.context)
                                    .map(|entity| FlashTarget::EntityAndScreen {
                                        entity,
                                        screen: screen.resolve(&player.context),
                                    }),
                            };
                            if let Some(target) = target {
                                flash_writer.write(RequestFlash {
                                    target,
                                    color: action.color,
                                    intensity: action.intensity * intensity,
                                    chromatic_aberration: action.chromatic_aberration * intensity,
                                    vignette: action.vignette * intensity,
                                    origin: action
                                        .use_context_origin
                                        .then_some(player.context.origin)
                                        .flatten(),
                                    attenuation: action.attenuation,
                                    duration_secs: action.duration_secs,
                                    easing: action.easing,
                                    clock: action.clock,
                                });
                            }
                        }
                        FeedbackAction::Hitstop(action) => {
                            emit_hitstop(&player.context, action, &mut hitstop_writer);
                        }
                        FeedbackAction::Rumble(action) => {
                            rumble_writer.write(RequestRumble {
                                target: action.target.resolve(&player.context),
                                low_frequency: action.low_frequency * intensity,
                                high_frequency: action.high_frequency * intensity,
                                duration_secs: action.duration_secs,
                                easing: action.easing,
                                clock: action.clock,
                            });
                        }
                        FeedbackAction::TimeScale(action) => {
                            emit_time_scale(&player.context, action, &mut time_scale_writer);
                        }
                        FeedbackAction::SquashStretch(action) => {
                            if let Some(target) = action.target.resolve(&player.context) {
                                let scaled_peak =
                                    Vec3::ONE + (action.peak_scale - Vec3::ONE) * intensity;
                                squash_writer.write(RequestSquashStretch {
                                    target,
                                    peak_scale: scaled_peak,
                                    mode: action.mode,
                                    duration_secs: action.duration_secs,
                                    easing: action.easing,
                                    clock: action.clock,
                                    stacking: action.stacking,
                                    direction: action
                                        .direction_from_context
                                        .then_some(player.context.direction),
                                    directional_magnitude: if action.direction_from_context {
                                        (scaled_peak - Vec3::ONE).length()
                                    } else {
                                        0.0
                                    },
                                });
                            }
                        }
                        FeedbackAction::Knockback(action) => {
                            if let Some(target) = action.target.resolve(&player.context) {
                                let direction = if action.direction_from_context
                                    && player.context.direction.length_squared() > f32::EPSILON
                                {
                                    player.context.direction.normalize_or_zero()
                                } else {
                                    Vec3::ZERO
                                };
                                knockback_writer.write(RequestKnockback {
                                    target,
                                    direction,
                                    force: action.force * intensity,
                                    duration_secs: action.duration_secs,
                                    easing: action.easing,
                                    clock: action.clock,
                                });
                            }
                        }
                        FeedbackAction::Hooks(action) => {
                            hook_writer.write(FeedbackHookTriggered {
                                recipe_name: player.recipe_name.clone(),
                                step_name: step.name.clone(),
                                step_index: player.fired_steps,
                                loop_index: player.completed_loops,
                                audio_cue: action.audio_cue.clone(),
                                particle_cue: action.particle_cue.clone(),
                                target: action
                                    .target
                                    .and_then(|target| target.resolve(&player.context)),
                                origin: action
                                    .use_context_origin
                                    .then_some(player.context.origin)
                                    .flatten(),
                            });
                        }
                    }
                }

                step_writer.write(FeedbackStepFired {
                    recipe_name: player.recipe_name.clone(),
                    step_name: step.name.clone(),
                    step_index: player.fired_steps,
                    loop_index: player.completed_loops,
                });
                player.fired_steps += 1;
            }

            if player.fired_steps < recipe.steps.len() {
                return true;
            }

            let recipe_duration = recipe.steps.last().map_or(0.0, |step| step.at_secs);
            match recipe.repeat {
                FeedbackRecipeRepeat::Once => return false,
                FeedbackRecipeRepeat::Times {
                    total_plays,
                    gap_secs,
                } => {
                    let total_plays = total_plays.max(1);
                    if player.completed_loops + 1 >= total_plays {
                        return false;
                    }

                    let repeat_after = recipe_duration + gap_secs.max(0.0);
                    if player.elapsed_secs < repeat_after {
                        return true;
                    }

                    player.completed_loops += 1;
                    player.elapsed_secs = (player.elapsed_secs - repeat_after).max(0.0);
                    player.fired_steps = 0;
                }
            }
        }
    });
}

fn emit_hitstop(
    context: &FeedbackContext,
    action: &RecipeHitstop,
    writer: &mut MessageWriter<RequestHitstop>,
) {
    match action.target {
        TimeScaleSelector::World => {
            writer.write(RequestHitstop {
                target: TimeScaleTarget::World,
                hold_frames: action.hold_frames,
                recovery_frames: action.recovery_frames,
                stacking: action.stacking,
                group_entities: Vec::new(),
            });
        }
        TimeScaleSelector::ContextTarget => {
            if let Some(target) = context.target {
                writer.write(RequestHitstop {
                    target: TimeScaleTarget::Entity(target),
                    hold_frames: action.hold_frames,
                    recovery_frames: action.recovery_frames,
                    stacking: action.stacking,
                    group_entities: Vec::new(),
                });
            }
        }
        TimeScaleSelector::ContextGroup => {
            writer.write(RequestHitstop {
                target: TimeScaleTarget::Group,
                hold_frames: action.hold_frames,
                recovery_frames: action.recovery_frames,
                stacking: action.stacking,
                group_entities: context.group.clone(),
            });
        }
        TimeScaleSelector::Specific(entity) => {
            writer.write(RequestHitstop {
                target: TimeScaleTarget::Entity(entity),
                hold_frames: action.hold_frames,
                recovery_frames: action.recovery_frames,
                stacking: action.stacking,
                group_entities: Vec::new(),
            });
        }
    };
}

fn emit_time_scale(
    context: &FeedbackContext,
    action: &RecipeTimeScale,
    writer: &mut MessageWriter<RequestTimeScale>,
) {
    match action.target {
        TimeScaleSelector::World => {
            writer.write(RequestTimeScale {
                target: TimeScaleTarget::World,
                scale: action.scale,
                ramp_in_secs: action.ramp_in_secs,
                hold_secs: action.hold_secs,
                ramp_out_secs: action.ramp_out_secs,
                easing: action.easing,
                priority: action.priority,
                group_entities: Vec::new(),
            });
        }
        TimeScaleSelector::ContextTarget => {
            if let Some(target) = context.target {
                writer.write(RequestTimeScale {
                    target: TimeScaleTarget::Entity(target),
                    scale: action.scale,
                    ramp_in_secs: action.ramp_in_secs,
                    hold_secs: action.hold_secs,
                    ramp_out_secs: action.ramp_out_secs,
                    easing: action.easing,
                    priority: action.priority,
                    group_entities: Vec::new(),
                });
            }
        }
        TimeScaleSelector::ContextGroup => {
            writer.write(RequestTimeScale {
                target: TimeScaleTarget::Group,
                scale: action.scale,
                ramp_in_secs: action.ramp_in_secs,
                hold_secs: action.hold_secs,
                ramp_out_secs: action.ramp_out_secs,
                easing: action.easing,
                priority: action.priority,
                group_entities: context.group.clone(),
            });
        }
        TimeScaleSelector::Specific(entity) => {
            writer.write(RequestTimeScale {
                target: TimeScaleTarget::Entity(entity),
                scale: action.scale,
                ramp_in_secs: action.ramp_in_secs,
                hold_secs: action.hold_secs,
                ramp_out_secs: action.ramp_out_secs,
                easing: action.easing,
                priority: action.priority,
                group_entities: Vec::new(),
            });
        }
    };
}

#[cfg(test)]
#[path = "recipe_tests.rs"]
mod tests;
