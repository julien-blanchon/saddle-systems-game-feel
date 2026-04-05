mod channels;
mod config;
mod flash;
mod knockback;
mod messages;
mod punch;
mod recipe;
mod rumble;
mod shake;
mod squash;
mod time_scale;
mod tween;

pub use channels::GameFeelChannels;
pub use config::{
    DistanceAttenuation, EffectTimeDomain, GameFeelConfig, GameFeelDiagnostics, GameFeelToggles,
    ScreenPulsePresentation,
};
pub use flash::{FlashOutput, FlashTarget, ScreenPulseListener, ScreenPulseOutput};
pub use knockback::{KnockbackReceiver, KnockbackState};
pub use messages::{
    AddTrauma, ListenerTarget, PlayFeedbackRecipe, RequestCameraImpulse, RequestFlash,
    RequestHitstop, RequestKnockback, RequestRumble, RequestSplitHitstop, RequestSquashStretch,
    RequestTimeScale,
};
pub use punch::{ImpulseSpace, PunchListener, PunchProfile, PunchState, SpringSettings};
pub use recipe::{
    EntitySelector, FeedbackAction, FeedbackCondition, FeedbackContext, FeedbackHookTriggered,
    FeedbackRecipe, FeedbackRecipeLibrary, FeedbackRecipeRepeat, FeedbackStep, FeedbackStepFired,
    ListenerSelector, RecipeFlash, RecipeFlashTarget, RecipeHitstop, RecipeHooks, RecipeImpulse,
    RecipeKnockback, RecipeRumble, RecipeSquashStretch, RecipeTimeScale, RecipeTrauma,
    TimeScaleSelector,
};
pub use rumble::{RumbleListener, RumbleOutput};
pub use shake::{ShakeAccessibility, ShakeBudget, ShakeListener, ShakeProfile, ShakeState};
pub use squash::{ScaleEffectMode, ScaleStacking, SquashStretchState};
pub use time_scale::{
    EntityTimeScale, GlobalTimeScale, HitstopStacking, IgnoreGlobalTimeScale, IgnoreHitstop,
    TimeScaleTarget, resolve_effective_time_scale,
};
pub use tween::{AttackSustainDecay, Tween, TweenRepeat, TweenSample};

use bevy::{
    app::PostStartup,
    camera::Projection,
    ecs::{intern::Interned, message::Messages, schedule::ScheduleLabel},
    prelude::*,
};

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum GameFeelSystems {
    ProcessRequests,
    UpdateSimulation,
    ApplyOutputs,
    Cleanup,
}

#[derive(Resource, Default)]
pub(crate) struct GameFeelRuntimeState {
    pub active: bool,
}

#[derive(Component, Debug, Default)]
struct PresentedTransformState {
    original_translation: Vec3,
    original_rotation: Quat,
    original_scale: Vec3,
    active_last_frame: bool,
}

#[derive(Component, Debug, Default)]
struct PresentedProjectionState {
    original_fov: f32,
    active_last_frame: bool,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

pub struct GameFeelPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl GameFeelPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for GameFeelPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for GameFeelPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.init_resource::<GameFeelRuntimeState>()
            .init_resource::<GameFeelConfig>()
            .init_resource::<GameFeelDiagnostics>()
            .init_resource::<GameFeelToggles>()
            .init_resource::<GlobalTimeScale>()
            .init_resource::<ShakeAccessibility>()
            .init_resource::<shake::PendingShakeQueue>()
            .init_resource::<punch::PendingPunchQueue>()
            .init_resource::<time_scale::WorldTimeScaleRuntime>()
            .init_resource::<recipe::RecipeRuntime>()
            .init_resource::<FeedbackRecipeLibrary>()
            .add_message::<AddTrauma>()
            .add_message::<RequestCameraImpulse>()
            .add_message::<RequestTimeScale>()
            .add_message::<RequestHitstop>()
            .add_message::<RequestSplitHitstop>()
            .add_message::<RequestFlash>()
            .add_message::<RequestRumble>()
            .add_message::<RequestSquashStretch>()
            .add_message::<RequestKnockback>()
            .add_message::<PlayFeedbackRecipe>()
            .add_message::<FeedbackStepFired>()
            .add_message::<FeedbackHookTriggered>()
            .register_type::<AddTrauma>()
            .register_type::<AttackSustainDecay>()
            .register_type::<DistanceAttenuation>()
            .register_type::<EffectTimeDomain>()
            .register_type::<EntitySelector>()
            .register_type::<EntityTimeScale>()
            .register_type::<FeedbackAction>()
            .register_type::<FeedbackCondition>()
            .register_type::<FeedbackContext>()
            .register_type::<FeedbackHookTriggered>()
            .register_type::<FeedbackRecipe>()
            .register_type::<FeedbackRecipeLibrary>()
            .register_type::<FeedbackRecipeRepeat>()
            .register_type::<FeedbackStep>()
            .register_type::<FeedbackStepFired>()
            .register_type::<FlashOutput>()
            .register_type::<FlashTarget>()
            .register_type::<GameFeelChannels>()
            .register_type::<GameFeelConfig>()
            .register_type::<GameFeelDiagnostics>()
            .register_type::<GlobalTimeScale>()
            .register_type::<HitstopStacking>()
            .register_type::<IgnoreGlobalTimeScale>()
            .register_type::<IgnoreHitstop>()
            .register_type::<GameFeelToggles>()
            .register_type::<ImpulseSpace>()
            .register_type::<KnockbackReceiver>()
            .register_type::<KnockbackState>()
            .register_type::<ListenerSelector>()
            .register_type::<ListenerTarget>()
            .register_type::<PlayFeedbackRecipe>()
            .register_type::<PunchListener>()
            .register_type::<PunchProfile>()
            .register_type::<PunchState>()
            .register_type::<RecipeFlash>()
            .register_type::<RecipeFlashTarget>()
            .register_type::<RecipeHooks>()
            .register_type::<RecipeHitstop>()
            .register_type::<RecipeKnockback>()
            .register_type::<RecipeImpulse>()
            .register_type::<RecipeRumble>()
            .register_type::<RecipeSquashStretch>()
            .register_type::<RecipeTimeScale>()
            .register_type::<RecipeTrauma>()
            .register_type::<RequestCameraImpulse>()
            .register_type::<RequestFlash>()
            .register_type::<RequestHitstop>()
            .register_type::<RequestKnockback>()
            .register_type::<RequestRumble>()
            .register_type::<RequestSplitHitstop>()
            .register_type::<RequestSquashStretch>()
            .register_type::<RequestTimeScale>()
            .register_type::<RumbleListener>()
            .register_type::<RumbleOutput>()
            .register_type::<ScaleEffectMode>()
            .register_type::<ScaleStacking>()
            .register_type::<ScreenPulsePresentation>()
            .register_type::<ScreenPulseListener>()
            .register_type::<ScreenPulseOutput>()
            .register_type::<ShakeAccessibility>()
            .register_type::<ShakeBudget>()
            .register_type::<ShakeListener>()
            .register_type::<ShakeProfile>()
            .register_type::<ShakeState>()
            .register_type::<SpringSettings>()
            .register_type::<SquashStretchState>()
            .register_type::<TimeScaleSelector>()
            .register_type::<TimeScaleTarget>()
            .register_type::<Tween>()
            .register_type::<TweenRepeat>()
            .register_type::<TweenSample>()
            .add_systems(self.activate_schedule, activate_runtime)
            .add_systems(self.deactivate_schedule, deactivate_runtime);

        let use_main_update_adapters = self.update_schedule == Update.intern();

        if use_main_update_adapters {
            app.configure_sets(PostUpdate, GameFeelSystems::ApplyOutputs);
            app.add_systems(
                PreUpdate,
                (
                    restore_presented_transforms,
                    restore_presented_projections,
                    flash::restore_presented_sprites,
                    flash::restore_presented_chromatic,
                ),
            );
        } else {
            app.configure_sets(
                self.update_schedule,
                (
                    GameFeelSystems::ProcessRequests,
                    GameFeelSystems::UpdateSimulation,
                    GameFeelSystems::ApplyOutputs,
                    GameFeelSystems::Cleanup,
                )
                    .chain(),
            );
            app.add_systems(
                self.update_schedule,
                (
                    restore_presented_transforms,
                    restore_presented_projections,
                    flash::restore_presented_sprites,
                    flash::restore_presented_chromatic,
                )
                    .before(GameFeelSystems::ProcessRequests),
            );
        }

        if use_main_update_adapters {
            app.configure_sets(
                self.update_schedule,
                (
                    GameFeelSystems::ProcessRequests,
                    GameFeelSystems::UpdateSimulation,
                    GameFeelSystems::Cleanup,
                )
                    .chain(),
            );
        }

        app.add_systems(
            self.update_schedule,
            (
                (
                    time_scale::advance_global_clock,
                    recipe::update_recipe_cooldowns,
                    recipe::start_recipe_players,
                    recipe::advance_recipe_players,
                    process_split_hitstop_requests,
                    process_time_scale_requests,
                    process_hitstop_requests,
                    process_trauma_requests,
                    process_punch_requests,
                    process_flash_requests,
                    process_rumble_requests,
                    process_squash_requests,
                    knockback::process_knockback_requests,
                )
                    .chain()
                    .in_set(GameFeelSystems::ProcessRequests),
                (
                    time_scale::resolve_global_time_scale,
                    time_scale::update_local_time_scales,
                    shake::advance_shake_queue,
                    shake::prepare_shake_states,
                    shake::apply_due_shake_events,
                    shake::update_shake_outputs,
                    punch::advance_punch_queue,
                    punch::apply_due_punch_events,
                    punch::update_punch_states,
                    flash::update_entity_flash_outputs,
                    flash::update_screen_pulse_outputs,
                    rumble::update_rumble_outputs,
                    squash::update_scale_outputs,
                    knockback::update_knockback_outputs,
                )
                    .chain()
                    .in_set(GameFeelSystems::UpdateSimulation),
                (
                    cleanup_empty_entity_flash_runtimes,
                    cleanup_empty_screen_pulse_runtimes,
                    cleanup_empty_rumble_runtimes,
                    cleanup_empty_scale_runtimes,
                    knockback::cleanup_knockback,
                    flash::cleanup_orphaned_overlays,
                    flash::ensure_screen_overlays.run_if(runtime_is_active),
                )
                    .chain()
                    .in_set(GameFeelSystems::Cleanup),
            )
                .run_if(runtime_is_active),
        );

        let apply_outputs = (
            apply_transform_outputs,
            apply_projection_outputs,
            flash::apply_sprite_flash_outputs,
            flash::apply_chromatic_outputs,
            flash::apply_screen_overlay_outputs,
        )
            .chain()
            .in_set(GameFeelSystems::ApplyOutputs)
            .run_if(runtime_is_active);

        if use_main_update_adapters {
            app.add_systems(PostUpdate, apply_outputs);
        } else {
            app.add_systems(self.update_schedule, apply_outputs);
        }
    }
}

pub(crate) fn runtime_is_active(runtime: Res<GameFeelRuntimeState>) -> bool {
    runtime.active
}

pub(crate) fn activate_runtime(mut runtime: ResMut<GameFeelRuntimeState>) {
    runtime.active = true;
}

pub(crate) fn deactivate_runtime(mut runtime: ResMut<GameFeelRuntimeState>) {
    runtime.active = false;
}

fn process_trauma_requests(world: &mut World) {
    let requests: Vec<AddTrauma> = world
        .resource_mut::<Messages<AddTrauma>>()
        .drain()
        .collect();
    if requests.is_empty() {
        return;
    }

    let mut pending = Vec::new();
    let mut query = world.query::<(
        Entity,
        &ShakeListener,
        Option<&Transform>,
        Option<&GlobalTransform>,
    )>();
    for request in requests {
        for (listener_entity, listener, transform, global_transform) in query.iter(world) {
            if !listener_matches(listener_entity, listener.channels, request.target) {
                continue;
            }

            let profile = request
                .profile_override
                .clone()
                .unwrap_or_else(|| listener.profile.clone());
            let distance = listener_distance(request.origin, transform, global_transform);
            let trauma = shake::attenuated_trauma(request.trauma, request.attenuation, distance);
            if trauma <= 0.0001 {
                continue;
            }

            pending.push((
                listener_entity,
                trauma.clamp(0.0, 1.0),
                request.directional_bias,
                profile,
                shake::propagation_delay_secs(request.propagation_speed, distance),
            ));
        }
    }

    let mut queue = world.resource_mut::<shake::PendingShakeQueue>();
    for (listener, trauma, directional_bias, profile, due_in_secs) in pending {
        shake::queue_shake_event(
            &mut queue,
            listener,
            trauma,
            directional_bias,
            profile,
            due_in_secs,
        );
    }
}

fn process_punch_requests(world: &mut World) {
    let requests: Vec<RequestCameraImpulse> = world
        .resource_mut::<Messages<RequestCameraImpulse>>()
        .drain()
        .collect();
    if requests.is_empty() {
        return;
    }

    let mut pending = Vec::new();
    let mut query = world.query::<(Entity, &PunchListener, &Transform, Option<&GlobalTransform>)>();
    for request in requests {
        for (listener_entity, listener, transform, global_transform) in query.iter(world) {
            if !listener_matches(listener_entity, listener.channels, request.target) {
                continue;
            }

            let distance = listener_distance(request.origin, Some(transform), global_transform);
            let factor = punch::attenuation(1.0, request.attenuation, distance).clamp(0.0, 1.0);
            if factor <= 0.0001 {
                continue;
            }

            pending.push((
                listener_entity,
                request.translation * factor,
                request.rotation * factor,
                request.fov * factor,
                request.space,
                request
                    .profile_override
                    .clone()
                    .unwrap_or_else(|| listener.profile.clone()),
                punch::propagation_delay(request.propagation_speed, distance),
            ));
        }
    }

    let mut queue = world.resource_mut::<punch::PendingPunchQueue>();
    for (listener, translation, rotation, fov, space, profile, due_in_secs) in pending {
        punch::queue_punch_event(
            &mut queue,
            listener,
            translation,
            rotation,
            fov,
            space,
            profile,
            due_in_secs,
        );
    }
}

fn process_time_scale_requests(world: &mut World) {
    let requests: Vec<RequestTimeScale> = world
        .resource_mut::<Messages<RequestTimeScale>>()
        .drain()
        .collect();

    for request in requests {
        if (request.scale - 1.0).abs() <= f32::EPSILON
            && request.ramp_in_secs <= 0.0
            && request.hold_secs <= 0.0
            && request.ramp_out_secs <= 0.0
        {
            continue;
        }

        match request.target {
            TimeScaleTarget::World => {
                let mut runtime = world.resource_mut::<time_scale::WorldTimeScaleRuntime>();
                runtime.ramps.push(time_scale::TimeScaleRamp {
                    scale: request.scale,
                    ramp_in_secs: request.ramp_in_secs,
                    hold_secs: request.hold_secs,
                    ramp_out_secs: request.ramp_out_secs,
                    easing: request.easing,
                    priority: request.priority,
                    elapsed_secs: 0.0,
                });
            }
            TimeScaleTarget::Entity(entity) => {
                add_local_time_scale_ramp(world, entity, &request);
            }
            TimeScaleTarget::Group => {
                for &entity in &request.group_entities {
                    add_local_time_scale_ramp(world, entity, &request);
                }
            }
        }
    }
}

fn process_hitstop_requests(world: &mut World) {
    let requests: Vec<RequestHitstop> = world
        .resource_mut::<Messages<RequestHitstop>>()
        .drain()
        .collect();

    for request in requests {
        if request.hold_frames == 0 && request.recovery_frames == 0 {
            continue;
        }

        match request.target {
            TimeScaleTarget::World => {
                let mut runtime = world.resource_mut::<time_scale::WorldTimeScaleRuntime>();
                time_scale::apply_hitstop_stacking(
                    &mut runtime.hitstop,
                    request.hold_frames,
                    request.recovery_frames,
                    request.stacking,
                );
            }
            TimeScaleTarget::Entity(entity) => {
                add_local_hitstop(world, entity, &request);
            }
            TimeScaleTarget::Group => {
                for &entity in &request.group_entities {
                    add_local_hitstop(world, entity, &request);
                }
            }
        }
    }
}

fn process_split_hitstop_requests(world: &mut World) {
    let requests: Vec<RequestSplitHitstop> = world
        .resource_mut::<Messages<RequestSplitHitstop>>()
        .drain()
        .collect();
    if requests.is_empty() {
        return;
    }

    let mut messages = world.resource_mut::<Messages<RequestHitstop>>();
    for request in requests {
        messages.write(RequestHitstop {
            target: TimeScaleTarget::Entity(request.attacker),
            hold_frames: request.attacker_frames,
            recovery_frames: request.recovery_frames,
            stacking: request.stacking,
            group_entities: Vec::new(),
        });
        messages.write(RequestHitstop {
            target: TimeScaleTarget::Entity(request.target),
            hold_frames: request.target_frames,
            recovery_frames: request.recovery_frames,
            stacking: request.stacking,
            group_entities: Vec::new(),
        });
    }
}

fn process_flash_requests(world: &mut World) {
    let requests: Vec<RequestFlash> = world
        .resource_mut::<Messages<RequestFlash>>()
        .drain()
        .collect();
    if requests.is_empty() {
        return;
    }

    let mut screen_query = world.query::<(
        Entity,
        &ScreenPulseListener,
        Option<&Transform>,
        Option<&GlobalTransform>,
    )>();

    for request in requests {
        match request.target {
            FlashTarget::Entity(entity) => add_entity_flash(world, entity, &request),
            FlashTarget::Screen(target) => {
                let listeners =
                    collect_screen_pulse_targets(world, &mut screen_query, target, &request);
                for (listener, factor) in listeners {
                    add_screen_pulse(world, listener, &request, factor);
                }
            }
            FlashTarget::EntityAndScreen { entity, screen } => {
                add_entity_flash(world, entity, &request);
                let listeners =
                    collect_screen_pulse_targets(world, &mut screen_query, screen, &request);
                for (listener, factor) in listeners {
                    add_screen_pulse(world, listener, &request, factor);
                }
            }
        }
    }
}

fn process_rumble_requests(world: &mut World) {
    let requests: Vec<RequestRumble> = world
        .resource_mut::<Messages<RequestRumble>>()
        .drain()
        .collect();
    if requests.is_empty() {
        return;
    }

    let mut query = world.query::<(Entity, &RumbleListener)>();
    for request in requests {
        let mut listeners = Vec::new();
        for (listener_entity, listener) in query.iter(world) {
            if !listener_matches(listener_entity, listener.channels, request.target) {
                continue;
            }
            listeners.push(listener_entity);
        }

        for listener in listeners {
            add_rumble_pulse(world, listener, &request);
        }
    }
}

fn process_squash_requests(world: &mut World) {
    let requests: Vec<RequestSquashStretch> = world
        .resource_mut::<Messages<RequestSquashStretch>>()
        .drain()
        .collect();

    for request in requests {
        let Ok(mut entity) = world.get_entity_mut(request.target) else {
            continue;
        };

        if !entity.contains::<SquashStretchState>() {
            entity.insert(SquashStretchState::default());
        }
        if !entity.contains::<squash::ScaleEffectRuntime>() {
            entity.insert(squash::ScaleEffectRuntime::default());
        }

        let directional = request.direction.map_or(Vec3::ONE, |direction| {
            squash::directional_scale(direction, request.directional_magnitude)
        });
        let peak_scale = request.peak_scale * directional;

        let mut runtime = entity
            .get_mut::<squash::ScaleEffectRuntime>()
            .expect("scale runtime inserted above");
        runtime.stacking = request.stacking;
        runtime.effects.push(squash::ActiveScaleEffect {
            peak_scale,
            mode: request.mode,
            tween: squash::default_scale_tween(request.duration_secs, request.easing),
            clock: request.clock,
            elapsed_secs: 0.0,
        });
    }
}

fn restore_presented_transforms(mut query: Query<(&mut Transform, &mut PresentedTransformState)>) {
    for (mut transform, mut state) in &mut query {
        if state.active_last_frame {
            transform.translation = state.original_translation;
            transform.rotation = state.original_rotation;
            transform.scale = state.original_scale;
            state.active_last_frame = false;
        }
    }
}

fn apply_transform_outputs(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            Option<&mut PresentedTransformState>,
            Option<&ShakeState>,
            Option<&PunchState>,
            Option<&SquashStretchState>,
        ),
        Or<(
            With<ShakeState>,
            With<PunchState>,
            With<SquashStretchState>,
            With<PresentedTransformState>,
        )>,
    >,
) {
    for (entity, mut transform, state, shake, punch, squash) in &mut query {
        let local_translation = shake.map_or(Vec3::ZERO, |state| state.translation_offset)
            + punch.map_or(Vec3::ZERO, |state| state.local_translation_offset);
        let world_translation = punch.map_or(Vec3::ZERO, |state| state.world_translation_offset);
        let rotation_offset = shake.map_or(Vec3::ZERO, |state| state.rotation_offset)
            + punch.map_or(Vec3::ZERO, |state| state.rotation_offset);
        let scale_multiplier = squash.map_or(Vec3::ONE, |state| state.scale_multiplier);
        let active = local_translation.length_squared() > 1.0e-8
            || world_translation.length_squared() > 1.0e-8
            || rotation_offset.length_squared() > 1.0e-8
            || (scale_multiplier - Vec3::ONE).length_squared() > 1.0e-8;
        if !active {
            continue;
        }

        let original_translation = transform.translation;
        let original_rotation = transform.rotation;
        let original_scale = transform.scale;
        let world_offset = original_rotation * local_translation + world_translation;
        let rotation_delta = Quat::from_euler(
            EulerRot::XYZ,
            rotation_offset.x,
            rotation_offset.y,
            rotation_offset.z,
        );

        transform.translation = original_translation + world_offset;
        transform.rotation = original_rotation * rotation_delta;
        transform.scale = original_scale * scale_multiplier;

        match state {
            Some(mut state) => {
                state.original_translation = original_translation;
                state.original_rotation = original_rotation;
                state.original_scale = original_scale;
                state.active_last_frame = true;
            }
            None => {
                commands.entity(entity).insert(PresentedTransformState {
                    original_translation,
                    original_rotation,
                    original_scale,
                    active_last_frame: true,
                });
            }
        }
    }
}

fn restore_presented_projections(
    mut query: Query<(&mut Projection, &mut PresentedProjectionState)>,
) {
    for (mut projection, mut state) in &mut query {
        if !state.active_last_frame {
            continue;
        }
        if let Projection::Perspective(perspective) = &mut *projection {
            perspective.fov = state.original_fov;
        }
        state.active_last_frame = false;
    }
}

fn apply_projection_outputs(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &PunchState,
        &mut Projection,
        Option<&mut PresentedProjectionState>,
    )>,
) {
    let min_fov = 10.0_f32.to_radians();
    let max_fov = 170.0_f32.to_radians();

    for (entity, punch, mut projection, state) in &mut query {
        if punch.fov_offset.abs() <= 1.0e-5 {
            continue;
        }

        let Projection::Perspective(perspective) = &mut *projection else {
            continue;
        };

        let original_fov = perspective.fov;
        perspective.fov = (perspective.fov + punch.fov_offset).clamp(min_fov, max_fov);

        match state {
            Some(mut state) => {
                state.original_fov = original_fov;
                state.active_last_frame = true;
            }
            None => {
                commands.entity(entity).insert(PresentedProjectionState {
                    original_fov,
                    active_last_frame: true,
                });
            }
        }
    }
}

fn cleanup_empty_entity_flash_runtimes(
    mut commands: Commands,
    query: Query<(Entity, &flash::EntityFlashRuntime)>,
) {
    for (entity, runtime) in &query {
        if runtime.effects.is_empty() {
            commands
                .entity(entity)
                .remove::<flash::EntityFlashRuntime>();
        }
    }
}

fn cleanup_empty_screen_pulse_runtimes(
    mut commands: Commands,
    query: Query<(Entity, &flash::ScreenPulseRuntime)>,
) {
    for (entity, runtime) in &query {
        if runtime.effects.is_empty() {
            commands
                .entity(entity)
                .remove::<flash::ScreenPulseRuntime>();
        }
    }
}

fn cleanup_empty_rumble_runtimes(
    mut commands: Commands,
    query: Query<(Entity, &rumble::RumbleRuntime)>,
) {
    for (entity, runtime) in &query {
        if runtime.effects.is_empty() {
            commands.entity(entity).remove::<rumble::RumbleRuntime>();
        }
    }
}

fn cleanup_empty_scale_runtimes(
    mut commands: Commands,
    query: Query<(Entity, &squash::ScaleEffectRuntime)>,
) {
    for (entity, runtime) in &query {
        if runtime.effects.is_empty() {
            commands
                .entity(entity)
                .remove::<squash::ScaleEffectRuntime>();
        }
    }
}

fn add_local_time_scale_ramp(world: &mut World, entity: Entity, request: &RequestTimeScale) {
    let Ok(mut entity) = world.get_entity_mut(entity) else {
        return;
    };

    if !entity.contains::<time_scale::LocalTimeScaleRuntime>() {
        entity.insert((
            time_scale::LocalTimeScaleRuntime::default(),
            EntityTimeScale::default(),
        ));
    }

    let mut runtime = entity
        .get_mut::<time_scale::LocalTimeScaleRuntime>()
        .expect("time scale runtime inserted above");
    runtime.ramps.push(time_scale::TimeScaleRamp {
        scale: request.scale,
        ramp_in_secs: request.ramp_in_secs,
        hold_secs: request.hold_secs,
        ramp_out_secs: request.ramp_out_secs,
        easing: request.easing,
        priority: request.priority,
        elapsed_secs: 0.0,
    });
}

fn add_local_hitstop(world: &mut World, entity: Entity, request: &RequestHitstop) {
    let Ok(mut entity) = world.get_entity_mut(entity) else {
        return;
    };
    if entity.contains::<IgnoreHitstop>() {
        return;
    }

    if !entity.contains::<time_scale::LocalTimeScaleRuntime>() {
        entity.insert((
            time_scale::LocalTimeScaleRuntime::default(),
            EntityTimeScale::default(),
        ));
    }

    let mut runtime = entity
        .get_mut::<time_scale::LocalTimeScaleRuntime>()
        .expect("time scale runtime inserted above");
    time_scale::apply_hitstop_stacking(
        &mut runtime.hitstop,
        request.hold_frames,
        request.recovery_frames,
        request.stacking,
    );
}

fn collect_screen_pulse_targets(
    world: &mut World,
    query: &mut QueryState<(
        Entity,
        &ScreenPulseListener,
        Option<&Transform>,
        Option<&GlobalTransform>,
    )>,
    target: ListenerTarget,
    request: &RequestFlash,
) -> Vec<(Entity, f32)> {
    let mut listeners = Vec::new();
    for (listener_entity, listener, transform, global_transform) in query.iter(world) {
        if !listener_matches(listener_entity, listener.channels, target) {
            continue;
        }
        let distance = listener_distance(request.origin, transform, global_transform);
        let factor = attenuation_factor(request.attenuation, distance);
        if factor <= 0.0001 {
            continue;
        }
        listeners.push((listener_entity, factor));
    }
    listeners
}

fn add_entity_flash(world: &mut World, entity: Entity, request: &RequestFlash) {
    let factor = entity_distance_factor(world, entity, request.origin, request.attenuation);
    if factor <= 0.0001 {
        return;
    }

    let Ok(mut entity) = world.get_entity_mut(entity) else {
        return;
    };
    if !entity.contains::<flash::EntityFlashRuntime>() {
        entity.insert((flash::EntityFlashRuntime::default(), FlashOutput::default()));
    }

    let mut runtime = entity
        .get_mut::<flash::EntityFlashRuntime>()
        .expect("flash runtime inserted above");
    runtime.effects.push(flash::ActiveEntityFlash {
        color: request.color,
        intensity: request.intensity * factor,
        tween: Tween {
            delay_secs: 0.0,
            duration_secs: request.duration_secs.max(f32::EPSILON),
            easing: request.easing,
            repeat: TweenRepeat::Once,
        },
        clock: request.clock,
        elapsed_secs: 0.0,
    });
}

fn add_screen_pulse(world: &mut World, entity: Entity, request: &RequestFlash, factor: f32) {
    let Ok(mut entity) = world.get_entity_mut(entity) else {
        return;
    };
    if !entity.contains::<flash::ScreenPulseRuntime>() {
        entity.insert((
            flash::ScreenPulseRuntime::default(),
            ScreenPulseOutput::default(),
        ));
    }

    let mut runtime = entity
        .get_mut::<flash::ScreenPulseRuntime>()
        .expect("screen pulse runtime inserted above");
    runtime.effects.push(flash::ActiveScreenPulse {
        color: request.color,
        flash_alpha: request.intensity * factor,
        chromatic_aberration: request.chromatic_aberration * factor,
        vignette: request.vignette * factor,
        tween: Tween {
            delay_secs: 0.0,
            duration_secs: request.duration_secs.max(f32::EPSILON),
            easing: request.easing,
            repeat: TweenRepeat::Once,
        },
        clock: request.clock,
        elapsed_secs: 0.0,
    });
}

fn add_rumble_pulse(world: &mut World, entity: Entity, request: &RequestRumble) {
    let Ok(mut entity) = world.get_entity_mut(entity) else {
        return;
    };
    if !entity.contains::<rumble::RumbleRuntime>() {
        entity.insert((rumble::RumbleRuntime::default(), RumbleOutput::default()));
    }

    let mut runtime = entity
        .get_mut::<rumble::RumbleRuntime>()
        .expect("rumble runtime inserted above");
    runtime.effects.push(rumble::ActiveRumblePulse {
        low_frequency: request.low_frequency.clamp(0.0, 1.0),
        high_frequency: request.high_frequency.clamp(0.0, 1.0),
        tween: Tween {
            delay_secs: 0.0,
            duration_secs: request.duration_secs.max(f32::EPSILON),
            easing: request.easing,
            repeat: TweenRepeat::Once,
        },
        clock: request.clock,
        elapsed_secs: 0.0,
    });
}

fn listener_matches(entity: Entity, channels: GameFeelChannels, target: ListenerTarget) -> bool {
    match target {
        ListenerTarget::All => true,
        ListenerTarget::Channels(target_channels) => channels.intersects(target_channels),
        ListenerTarget::Entity(target_entity) => entity == target_entity,
    }
}

fn listener_distance(
    origin: Option<Vec3>,
    transform: Option<&Transform>,
    global_transform: Option<&GlobalTransform>,
) -> f32 {
    let Some(origin) = origin else {
        return 0.0;
    };
    let Some(position) = transform_position(transform, global_transform) else {
        return 0.0;
    };
    position.distance(origin)
}

fn transform_position(
    transform: Option<&Transform>,
    global_transform: Option<&GlobalTransform>,
) -> Option<Vec3> {
    global_transform
        .map(GlobalTransform::translation)
        .or_else(|| transform.map(|transform| transform.translation))
}

fn attenuation_factor(attenuation: Option<DistanceAttenuation>, distance: f32) -> f32 {
    attenuation
        .map_or(1.0, |attenuation| attenuation.sample(distance))
        .clamp(0.0, 1.0)
}

fn entity_distance_factor(
    world: &World,
    entity: Entity,
    origin: Option<Vec3>,
    attenuation: Option<DistanceAttenuation>,
) -> f32 {
    let Some(origin) = origin else {
        return 1.0;
    };

    let position = world
        .get::<GlobalTransform>(entity)
        .map(GlobalTransform::translation)
        .or_else(|| {
            world
                .get::<Transform>(entity)
                .map(|transform| transform.translation)
        });
    position.map_or(1.0, |position| {
        attenuation_factor(attenuation, position.distance(origin))
    })
}

#[cfg(test)]
#[path = "plugin_tests.rs"]
mod plugin_tests;
