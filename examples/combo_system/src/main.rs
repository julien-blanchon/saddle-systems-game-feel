use saddle_systems_game_feel_example_support as support;

use bevy::prelude::*;
use saddle_systems_game_feel::{
    EffectTimeDomain, EntitySelector, FeedbackAction, FeedbackCondition, FeedbackContext,
    FeedbackHookTriggered, FeedbackRecipe, FeedbackRecipeLibrary, FeedbackRecipeRepeat,
    FeedbackStep, FeedbackStepFired, GameFeelPlugin, GameFeelSystems, ListenerSelector,
    PlayFeedbackRecipe, RecipeFlash, RecipeFlashTarget, RecipeHooks, RecipeImpulse,
    RecipeRumble, RecipeTrauma, presets,
};

#[derive(Resource)]
struct ComboTimer(Timer);

#[derive(Resource, Default)]
struct ComboHudState {
    cycles_started: u32,
    last_step: String,
    last_hook: String,
    last_loop: u32,
}

fn main() {
    let mut app = App::new();
    support::add_example_plugins(
        &mut app,
        "Game Feel Combo System",
        Color::srgb(0.04, 0.05, 0.08),
    );
    support::seed_example_pane(
        &mut app,
        support::ExampleFeelPane {
            interval_secs: 1.9,
            trauma_scale: 1.15,
            impulse_scale: 1.1,
            flash_scale: 1.15,
            chromatic_scale: 1.1,
            vignette_scale: 1.1,
            legacy_screen_fx: 0.0,
            ..default()
        },
    );
    app.insert_resource(combo_library());
    app.insert_resource(ComboTimer(Timer::from_seconds(1.9, TimerMode::Repeating)));
    app.insert_resource(ComboHudState::default());
    app.insert_resource(support::HudLabel(
        "Combo System\nA two-cycle three-hit finisher uses recipe hooks, conditional playback, and rumble outputs.".into(),
    ));
    app.add_plugins(GameFeelPlugin::default());
    app.add_systems(Startup, support::setup_2d_scene);
    app.add_systems(Update, support::advance_demo_motion);
    app.add_systems(
        Update,
        play_combo_cycle.before(GameFeelSystems::ProcessRequests),
    );
    app.add_systems(
        Update,
        (
            capture_combo_feedback,
            update_combo_label,
            support::update_hud,
        )
            .chain()
            .after(GameFeelSystems::ProcessRequests),
    );
    app.run();
}

fn combo_library() -> FeedbackRecipeLibrary {
    let mut library = presets::recipes::library();
    library.recipes.insert(
        "combo_showcase".into(),
        FeedbackRecipe {
            cooldown_secs: 0.0,
            condition: FeedbackCondition::RequiresListener,
            repeat: FeedbackRecipeRepeat::Times {
                total_plays: 2,
                gap_secs: 0.32,
            },
            steps: vec![
                FeedbackStep {
                    name: "opening_slash".into(),
                    at_secs: 0.0,
                    actions: vec![
                        FeedbackAction::Trauma(RecipeTrauma {
                            target: ListenerSelector::ContextListener,
                            trauma: 0.14,
                            directional_bias: Vec3::new(0.03, 0.0, 0.0),
                            use_context_origin: true,
                            attenuation: None,
                            propagation_speed: None,
                        }),
                        FeedbackAction::Flash(RecipeFlash {
                            target: RecipeFlashTarget::EntityAndScreen {
                                entity: EntitySelector::ContextTarget,
                                screen: ListenerSelector::ContextListener,
                            },
                            color: Color::srgb(1.0, 0.52, 0.34),
                            intensity: 0.52,
                            chromatic_aberration: 0.0,
                            vignette: 0.0,
                            use_context_origin: true,
                            attenuation: None,
                            duration_secs: 0.12,
                            easing: bevy::math::curve::easing::EaseFunction::QuadraticOut,
                            clock: EffectTimeDomain::GlobalScaled,
                        }),
                        FeedbackAction::CameraImpulse(RecipeImpulse {
                            target: ListenerSelector::ContextListener,
                            translation: Vec3::new(-0.06, 0.02, 0.0),
                            rotation: Vec3::new(0.0, 0.0, 0.015),
                            fov: 0.0,
                            use_context_origin: true,
                            attenuation: None,
                            propagation_speed: None,
                            space: saddle_systems_game_feel::ImpulseSpace::Local,
                        }),
                        FeedbackAction::Hooks(RecipeHooks {
                            audio_cue: Some("blade_whoosh_light".into()),
                            particle_cue: Some("orange_sparks_small".into()),
                            target: Some(EntitySelector::ContextTarget),
                            use_context_origin: true,
                        }),
                    ],
                },
                FeedbackStep {
                    name: "cross_cut".into(),
                    at_secs: 0.12,
                    actions: vec![
                        FeedbackAction::Trauma(RecipeTrauma {
                            target: ListenerSelector::ContextListener,
                            trauma: 0.18,
                            directional_bias: Vec3::new(-0.04, 0.01, 0.0),
                            use_context_origin: true,
                            attenuation: None,
                            propagation_speed: None,
                        }),
                        FeedbackAction::Flash(RecipeFlash {
                            target: RecipeFlashTarget::EntityAndScreen {
                                entity: EntitySelector::ContextTarget,
                                screen: ListenerSelector::ContextListener,
                            },
                            color: Color::srgb(0.98, 0.82, 0.34),
                            intensity: 0.62,
                            chromatic_aberration: 0.04,
                            vignette: 0.08,
                            use_context_origin: true,
                            attenuation: None,
                            duration_secs: 0.14,
                            easing: bevy::math::curve::easing::EaseFunction::QuadraticOut,
                            clock: EffectTimeDomain::GlobalScaled,
                        }),
                        FeedbackAction::Hooks(RecipeHooks {
                            audio_cue: Some("blade_whoosh_heavy".into()),
                            particle_cue: Some("yellow_sparks_medium".into()),
                            target: Some(EntitySelector::ContextTarget),
                            use_context_origin: true,
                        }),
                    ],
                },
                FeedbackStep {
                    name: "finisher".into(),
                    at_secs: 0.30,
                    actions: vec![
                        FeedbackAction::Trauma(RecipeTrauma {
                            target: ListenerSelector::ContextListener,
                            trauma: 0.26,
                            directional_bias: Vec3::new(0.08, -0.01, 0.0),
                            use_context_origin: true,
                            attenuation: None,
                            propagation_speed: None,
                        }),
                        FeedbackAction::CameraImpulse(RecipeImpulse {
                            target: ListenerSelector::ContextListener,
                            translation: Vec3::new(-0.12, 0.05, 0.0),
                            rotation: Vec3::new(0.0, 0.0, -0.03),
                            fov: 0.0,
                            use_context_origin: true,
                            attenuation: None,
                            propagation_speed: None,
                            space: saddle_systems_game_feel::ImpulseSpace::Local,
                        }),
                        FeedbackAction::Flash(RecipeFlash {
                            target: RecipeFlashTarget::EntityAndScreen {
                                entity: EntitySelector::ContextTarget,
                                screen: ListenerSelector::ContextListener,
                            },
                            color: Color::srgb(1.0, 0.34, 0.30),
                            intensity: 0.88,
                            chromatic_aberration: 0.08,
                            vignette: 0.16,
                            use_context_origin: true,
                            attenuation: None,
                            duration_secs: 0.2,
                            easing: bevy::math::curve::easing::EaseFunction::CubicOut,
                            clock: EffectTimeDomain::GlobalScaled,
                        }),
                        FeedbackAction::Rumble(RecipeRumble {
                            target: ListenerSelector::ContextListener,
                            low_frequency: 0.86,
                            high_frequency: 0.58,
                            duration_secs: 0.24,
                            easing: bevy::math::curve::easing::EaseFunction::CubicOut,
                            clock: EffectTimeDomain::GlobalScaled,
                        }),
                        FeedbackAction::Hooks(RecipeHooks {
                            audio_cue: Some("finisher_boom".into()),
                            particle_cue: Some("impact_ring_large".into()),
                            target: Some(EntitySelector::ContextTarget),
                            use_context_origin: true,
                        }),
                    ],
                },
            ],
        },
    );
    library
}

fn play_combo_cycle(
    time: Res<Time>,
    pane: Res<support::ExampleFeelPane>,
    mut timer: ResMut<ComboTimer>,
    mut state: ResMut<ComboHudState>,
    camera: Query<(Entity, &Transform), With<support::DemoCamera>>,
    target: Query<(Entity, &Transform), With<support::DemoTarget>>,
    mut recipes: MessageWriter<PlayFeedbackRecipe>,
) {
    timer.0.set_duration(std::time::Duration::from_secs_f32(
        pane.interval_secs.max(0.4),
    ));
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let Ok((camera, _)) = camera.single() else {
        return;
    };
    let Ok((target, transform)) = target.single() else {
        return;
    };

    state.cycles_started += 1;
    recipes.write(PlayFeedbackRecipe {
        name: "combo_showcase".into(),
        context: FeedbackContext {
            listener: Some(camera),
            target: Some(target),
            group: vec![target],
            origin: Some(transform.translation),
            direction: Vec3::new(1.0, -0.15, 0.0),
            channels: presets::channels::WEAPON,
            ..default()
        },
    });
}

fn capture_combo_feedback(
    mut state: ResMut<ComboHudState>,
    mut steps: MessageReader<FeedbackStepFired>,
    mut hooks: MessageReader<FeedbackHookTriggered>,
) {
    for step in steps.read() {
        if step.recipe_name != "combo_showcase" {
            continue;
        }

        state.last_step = format!(
            "{} (step {} / loop {})",
            step.step_name,
            step.step_index + 1,
            step.loop_index + 1
        );
        state.last_loop = step.loop_index + 1;
    }

    for hook in hooks.read() {
        if hook.recipe_name != "combo_showcase" {
            continue;
        }

        state.last_hook = format!(
            "{} | audio={} | particles={}",
            hook.step_name,
            hook.audio_cue.as_deref().unwrap_or("none"),
            hook.particle_cue.as_deref().unwrap_or("none")
        );
    }
}

fn update_combo_label(state: Res<ComboHudState>, mut label: ResMut<support::HudLabel>) {
    label.0 = format!(
        "Combo System\nTwo chained three-hit strings reuse a single recipe, fire hook cues, and finish with rumble.\nCycles started: {}\nLast step: {}\nLast hook: {}",
        state.cycles_started,
        if state.last_step.is_empty() {
            "waiting for combo".into()
        } else {
            state.last_step.clone()
        },
        if state.last_hook.is_empty() {
            "waiting for hook".into()
        } else {
            state.last_hook.clone()
        },
    );
}
