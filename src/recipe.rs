use crate::{
    channels::GameFeelChannels,
    config::{DistanceAttenuation, EffectTimeDomain, GameFeelDiagnostics},
    flash::FlashTarget,
    messages::{
        AddTrauma, ListenerTarget, PlayFeedbackRecipe, RequestCameraImpulse, RequestFlash,
        RequestHitstop, RequestRumble, RequestSquashStretch, RequestTimeScale,
    },
    punch::ImpulseSpace,
    time_scale::{HitstopStacking, TimeScaleTarget},
};
use bevy::{math::curve::easing::EaseFunction, prelude::*};
use std::collections::HashMap;

#[derive(Reflect, Clone, Debug, Default, PartialEq)]
pub struct FeedbackContext {
    pub listener: Option<Entity>,
    pub target: Option<Entity>,
    pub group: Vec<Entity>,
    pub origin: Option<Vec3>,
    pub direction: Vec3,
    pub channels: GameFeelChannels,
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
    pub duration_secs: f32,
    pub easing: EaseFunction,
    pub clock: EffectTimeDomain,
    pub direction_from_context: bool,
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

    #[must_use]
    pub fn with_builtin_presets() -> Self {
        let mut library = Self::empty();
        library.recipes.insert("light_hit".into(), light_hit());
        library
            .recipes
            .insert("heavy_impact".into(), heavy_impact());
        library.recipes.insert("explosion".into(), explosion());
        library.recipes.insert("reward_ping".into(), reward_ping());
        library
    }
}

impl Default for FeedbackRecipeLibrary {
    fn default() -> Self {
        Self::with_builtin_presets()
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

                for action in &step.actions {
                    match action {
                        FeedbackAction::Trauma(action) => {
                            trauma_writer.write(AddTrauma {
                                target: action.target.resolve(&player.context),
                                trauma: action.trauma,
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
                                } else {
                                    action.directional_bias
                                },
                                profile_override: None,
                            });
                        }
                        FeedbackAction::CameraImpulse(action) => {
                            impulse_writer.write(RequestCameraImpulse {
                                target: action.target.resolve(&player.context),
                                translation: action.translation,
                                rotation: action.rotation,
                                fov: action.fov,
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
                                    intensity: action.intensity,
                                    chromatic_aberration: action.chromatic_aberration,
                                    vignette: action.vignette,
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
                                low_frequency: action.low_frequency,
                                high_frequency: action.high_frequency,
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
                                squash_writer.write(RequestSquashStretch {
                                    target,
                                    peak_scale: action.peak_scale,
                                    mode: crate::squash::ScaleEffectMode::Relative,
                                    duration_secs: action.duration_secs,
                                    easing: action.easing,
                                    clock: action.clock,
                                    stacking: crate::squash::ScaleStacking::Multiply,
                                    direction: action
                                        .direction_from_context
                                        .then_some(player.context.direction),
                                    directional_magnitude: if action.direction_from_context {
                                        (action.peak_scale - Vec3::ONE).length()
                                    } else {
                                        0.0
                                    },
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

fn light_hit() -> FeedbackRecipe {
    FeedbackRecipe {
        cooldown_secs: 0.02,
        condition: FeedbackCondition::RequiresListener,
        repeat: FeedbackRecipeRepeat::Once,
        steps: vec![FeedbackStep {
            name: "impact".into(),
            at_secs: 0.0,
            actions: vec![
                FeedbackAction::Trauma(RecipeTrauma {
                    target: ListenerSelector::ContextListener,
                    trauma: 0.16,
                    directional_bias: Vec3::new(0.08, -0.02, 0.0),
                    use_context_origin: true,
                    attenuation: None,
                    propagation_speed: None,
                }),
                FeedbackAction::CameraImpulse(RecipeImpulse {
                    target: ListenerSelector::ContextListener,
                    translation: Vec3::new(-0.07, 0.03, 0.0),
                    rotation: Vec3::new(0.0, 0.0, -0.06),
                    fov: 0.0,
                    use_context_origin: false,
                    attenuation: None,
                    propagation_speed: None,
                    space: ImpulseSpace::Local,
                }),
                FeedbackAction::Flash(RecipeFlash {
                    target: RecipeFlashTarget::EntityAndScreen {
                        entity: EntitySelector::ContextTarget,
                        screen: ListenerSelector::ContextListener,
                    },
                    color: Color::WHITE,
                    intensity: 0.35,
                    chromatic_aberration: 0.03,
                    vignette: 0.08,
                    use_context_origin: false,
                    attenuation: None,
                    duration_secs: 0.12,
                    easing: EaseFunction::SineOut,
                    clock: EffectTimeDomain::Unscaled,
                }),
                FeedbackAction::Rumble(RecipeRumble {
                    target: ListenerSelector::ContextListener,
                    low_frequency: 0.30,
                    high_frequency: 0.45,
                    duration_secs: 0.12,
                    easing: EaseFunction::SineOut,
                    clock: EffectTimeDomain::Unscaled,
                }),
                FeedbackAction::Hitstop(RecipeHitstop {
                    target: TimeScaleSelector::ContextTarget,
                    hold_frames: 2,
                    recovery_frames: 2,
                    stacking: HitstopStacking::Refresh,
                }),
                FeedbackAction::Hooks(RecipeHooks {
                    audio_cue: Some("impact_light".into()),
                    particle_cue: Some("impact_spark".into()),
                    target: Some(EntitySelector::ContextTarget),
                    use_context_origin: true,
                }),
            ],
        }],
    }
}

fn heavy_impact() -> FeedbackRecipe {
    FeedbackRecipe {
        cooldown_secs: 0.08,
        condition: FeedbackCondition::RequiresListener,
        repeat: FeedbackRecipeRepeat::Once,
        steps: vec![
            FeedbackStep {
                name: "freeze".into(),
                at_secs: 0.0,
                actions: vec![
                    FeedbackAction::Hitstop(RecipeHitstop {
                        target: TimeScaleSelector::ContextTarget,
                        hold_frames: 5,
                        recovery_frames: 3,
                        stacking: HitstopStacking::Refresh,
                    }),
                    FeedbackAction::Trauma(RecipeTrauma {
                        target: ListenerSelector::ContextListener,
                        trauma: 0.34,
                        directional_bias: Vec3::new(0.22, -0.10, 0.0),
                        use_context_origin: true,
                        attenuation: Some(DistanceAttenuation {
                            inner_radius: 0.0,
                            outer_radius: 20.0,
                            exponent: 1.2,
                        }),
                        propagation_speed: None,
                    }),
                    FeedbackAction::CameraImpulse(RecipeImpulse {
                        target: ListenerSelector::ContextListener,
                        translation: Vec3::new(-0.20, 0.08, 0.0),
                        rotation: Vec3::new(0.0, 0.0, -0.12),
                        fov: 0.03,
                        use_context_origin: false,
                        attenuation: None,
                        propagation_speed: None,
                        space: ImpulseSpace::Local,
                    }),
                    FeedbackAction::Flash(RecipeFlash {
                        target: RecipeFlashTarget::EntityAndScreen {
                            entity: EntitySelector::ContextTarget,
                            screen: ListenerSelector::ContextListener,
                        },
                        color: Color::srgb(1.0, 0.95, 0.9),
                        intensity: 0.58,
                        chromatic_aberration: 0.08,
                        vignette: 0.20,
                        use_context_origin: false,
                        attenuation: None,
                        duration_secs: 0.18,
                        easing: EaseFunction::SineOut,
                        clock: EffectTimeDomain::Unscaled,
                    }),
                    FeedbackAction::Rumble(RecipeRumble {
                        target: ListenerSelector::ContextListener,
                        low_frequency: 0.85,
                        high_frequency: 0.55,
                        duration_secs: 0.20,
                        easing: EaseFunction::SineOut,
                        clock: EffectTimeDomain::Unscaled,
                    }),
                    FeedbackAction::SquashStretch(RecipeSquashStretch {
                        target: EntitySelector::ContextTarget,
                        peak_scale: Vec3::new(1.18, 0.82, 1.0),
                        duration_secs: 0.18,
                        easing: EaseFunction::BackOut,
                        clock: EffectTimeDomain::Unscaled,
                        direction_from_context: true,
                    }),
                    FeedbackAction::Hooks(RecipeHooks {
                        audio_cue: Some("impact_heavy".into()),
                        particle_cue: Some("impact_burst".into()),
                        target: Some(EntitySelector::ContextTarget),
                        use_context_origin: true,
                    }),
                ],
            },
            FeedbackStep {
                name: "slow_mo_tail".into(),
                at_secs: 0.04,
                actions: vec![FeedbackAction::TimeScale(RecipeTimeScale {
                    target: TimeScaleSelector::World,
                    scale: 0.55,
                    ramp_in_secs: 0.02,
                    hold_secs: 0.10,
                    ramp_out_secs: 0.16,
                    easing: EaseFunction::SineInOut,
                    priority: 1,
                })],
            },
        ],
    }
}

fn explosion() -> FeedbackRecipe {
    FeedbackRecipe {
        cooldown_secs: 0.15,
        condition: FeedbackCondition::RequiresListener,
        repeat: FeedbackRecipeRepeat::Once,
        steps: vec![
            FeedbackStep {
                name: "blast".into(),
                at_secs: 0.0,
                actions: vec![
                    FeedbackAction::Trauma(RecipeTrauma {
                        target: ListenerSelector::ContextListener,
                        trauma: 0.46,
                        directional_bias: Vec3::new(0.0, 0.0, -0.04),
                        use_context_origin: true,
                        attenuation: Some(DistanceAttenuation {
                            inner_radius: 0.0,
                            outer_radius: 30.0,
                            exponent: 1.4,
                        }),
                        propagation_speed: Some(16.0),
                    }),
                    FeedbackAction::CameraImpulse(RecipeImpulse {
                        target: ListenerSelector::ContextListener,
                        translation: Vec3::new(0.0, 0.14, 0.30),
                        rotation: Vec3::new(-0.05, 0.02, 0.03),
                        fov: 0.05,
                        use_context_origin: true,
                        attenuation: Some(DistanceAttenuation {
                            inner_radius: 0.0,
                            outer_radius: 30.0,
                            exponent: 1.2,
                        }),
                        propagation_speed: Some(16.0),
                        space: ImpulseSpace::World,
                    }),
                    FeedbackAction::Flash(RecipeFlash {
                        target: RecipeFlashTarget::Screen(ListenerSelector::ContextListener),
                        color: Color::srgb(1.0, 0.72, 0.40),
                        intensity: 0.45,
                        chromatic_aberration: 0.10,
                        vignette: 0.24,
                        use_context_origin: true,
                        attenuation: Some(DistanceAttenuation {
                            inner_radius: 0.0,
                            outer_radius: 24.0,
                            exponent: 1.0,
                        }),
                        duration_secs: 0.26,
                        easing: EaseFunction::SineOut,
                        clock: EffectTimeDomain::Unscaled,
                    }),
                    FeedbackAction::Rumble(RecipeRumble {
                        target: ListenerSelector::ContextListener,
                        low_frequency: 1.0,
                        high_frequency: 0.65,
                        duration_secs: 0.40,
                        easing: EaseFunction::SineOut,
                        clock: EffectTimeDomain::Unscaled,
                    }),
                    FeedbackAction::Hooks(RecipeHooks {
                        audio_cue: Some("explosion_heavy".into()),
                        particle_cue: Some("explosion_debris".into()),
                        target: None,
                        use_context_origin: true,
                    }),
                ],
            },
            FeedbackStep {
                name: "rumble_tail".into(),
                at_secs: 0.18,
                actions: vec![FeedbackAction::TimeScale(RecipeTimeScale {
                    target: TimeScaleSelector::World,
                    scale: 0.72,
                    ramp_in_secs: 0.0,
                    hold_secs: 0.16,
                    ramp_out_secs: 0.28,
                    easing: EaseFunction::SineInOut,
                    priority: 0,
                })],
            },
        ],
    }
}

fn reward_ping() -> FeedbackRecipe {
    FeedbackRecipe {
        cooldown_secs: 0.05,
        condition: FeedbackCondition::Always,
        repeat: FeedbackRecipeRepeat::Once,
        steps: vec![FeedbackStep {
            name: "pulse".into(),
            at_secs: 0.0,
            actions: vec![
                FeedbackAction::Flash(RecipeFlash {
                    target: RecipeFlashTarget::Screen(ListenerSelector::ContextListener),
                    color: Color::srgb(1.0, 0.95, 0.4),
                    intensity: 0.25,
                    chromatic_aberration: 0.02,
                    vignette: 0.05,
                    use_context_origin: false,
                    attenuation: None,
                    duration_secs: 0.18,
                    easing: EaseFunction::SineOut,
                    clock: EffectTimeDomain::Unscaled,
                }),
                FeedbackAction::TimeScale(RecipeTimeScale {
                    target: TimeScaleSelector::World,
                    scale: 0.82,
                    ramp_in_secs: 0.0,
                    hold_secs: 0.04,
                    ramp_out_secs: 0.12,
                    easing: EaseFunction::SineInOut,
                    priority: 0,
                }),
                FeedbackAction::Hooks(RecipeHooks {
                    audio_cue: Some("reward_ping".into()),
                    particle_cue: Some("reward_sparkle".into()),
                    target: Some(EntitySelector::ContextTarget),
                    use_context_origin: true,
                }),
            ],
        }],
    }
}

#[cfg(test)]
#[path = "recipe_tests.rs"]
mod tests;
