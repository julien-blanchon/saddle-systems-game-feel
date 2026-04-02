use super::*;
use crate::{
    AddTrauma, FeedbackStepFired, GameFeelDiagnostics, GlobalTimeScale, PlayFeedbackRecipe,
    RequestCameraImpulse, RequestFlash, RequestHitstop, RequestSquashStretch, RequestTimeScale,
};
use bevy::math::curve::easing::EaseFunction;

#[test]
fn default_library_contains_builtin_presets() {
    let library = FeedbackRecipeLibrary::default();
    assert!(library.recipes.contains_key("light_hit"));
    assert!(library.recipes.contains_key("heavy_impact"));
    assert!(library.recipes.contains_key("explosion"));
    assert!(library.recipes.contains_key("reward_ping"));
}

#[test]
fn recipe_step_emits_requests_on_first_frame() {
    let mut app = App::new();
    app.init_resource::<GlobalTimeScale>();
    app.init_resource::<GameFeelDiagnostics>();
    app.init_resource::<FeedbackRecipeLibrary>();
    app.init_resource::<RecipeRuntime>();
    app.add_message::<AddTrauma>();
    app.add_message::<RequestCameraImpulse>();
    app.add_message::<RequestFlash>();
    app.add_message::<RequestHitstop>();
    app.add_message::<RequestTimeScale>();
    app.add_message::<RequestSquashStretch>();
    app.add_message::<PlayFeedbackRecipe>();
    app.add_message::<FeedbackStepFired>();
    app.add_systems(
        Update,
        (
            update_recipe_cooldowns,
            start_recipe_players,
            advance_recipe_players,
        )
            .chain(),
    );

    let listener = app.world_mut().spawn_empty().id();
    let target = app.world_mut().spawn_empty().id();
    app.world_mut()
        .resource_mut::<Messages<PlayFeedbackRecipe>>()
        .write(PlayFeedbackRecipe {
            name: "light_hit".into(),
            context: FeedbackContext {
                listener: Some(listener),
                target: Some(target),
                group: vec![target],
                channels: GameFeelChannels::WEAPON,
                ..default()
            },
        });

    app.update();

    assert!(!app.world().resource::<Messages<AddTrauma>>().is_empty());
    assert!(
        !app.world()
            .resource::<Messages<RequestCameraImpulse>>()
            .is_empty()
    );
    assert!(!app.world().resource::<Messages<RequestFlash>>().is_empty());
    assert!(
        !app.world()
            .resource::<Messages<RequestHitstop>>()
            .is_empty()
    );
    assert!(
        !app.world()
            .resource::<Messages<FeedbackStepFired>>()
            .is_empty()
    );
}

#[test]
fn recipe_cooldown_prevents_duplicate_players_until_time_advances() {
    let mut app = App::new();
    app.init_resource::<GlobalTimeScale>();
    app.init_resource::<FeedbackRecipeLibrary>();
    app.init_resource::<RecipeRuntime>();
    app.add_message::<PlayFeedbackRecipe>();
    app.add_systems(
        Update,
        (update_recipe_cooldowns, start_recipe_players).chain(),
    );

    app.world_mut()
        .resource_mut::<Messages<PlayFeedbackRecipe>>()
        .write(PlayFeedbackRecipe::new("heavy_impact"));
    app.update();
    assert_eq!(app.world().resource::<RecipeRuntime>().players.len(), 1);

    app.world_mut()
        .resource_mut::<Messages<PlayFeedbackRecipe>>()
        .write(PlayFeedbackRecipe::new("heavy_impact"));
    app.update();
    assert_eq!(
        app.world().resource::<RecipeRuntime>().players.len(),
        1,
        "cooldown should reject a second immediate trigger"
    );

    app.world_mut()
        .resource_mut::<GlobalTimeScale>()
        .elapsed_unscaled_secs = 0.09;
    app.world_mut()
        .resource_mut::<Messages<PlayFeedbackRecipe>>()
        .write(PlayFeedbackRecipe::new("heavy_impact"));
    app.update();
    assert_eq!(app.world().resource::<RecipeRuntime>().players.len(), 2);
}

#[test]
fn delayed_steps_wait_until_their_scheduled_time() {
    let mut app = App::new();
    app.init_resource::<GlobalTimeScale>();
    app.init_resource::<GameFeelDiagnostics>();
    app.insert_resource(FeedbackRecipeLibrary {
        recipes: [(
            "timed".into(),
            FeedbackRecipe {
                cooldown_secs: 0.0,
                steps: vec![
                    FeedbackStep {
                        name: "instant".into(),
                        at_secs: 0.0,
                        actions: vec![FeedbackAction::Trauma(RecipeTrauma {
                            target: ListenerSelector::All,
                            trauma: 0.2,
                            directional_bias: Vec3::ZERO,
                            use_context_origin: false,
                            attenuation: None,
                            propagation_speed: None,
                        })],
                    },
                    FeedbackStep {
                        name: "delayed".into(),
                        at_secs: 0.2,
                        actions: vec![FeedbackAction::TimeScale(RecipeTimeScale {
                            target: TimeScaleSelector::World,
                            scale: 0.5,
                            ramp_in_secs: 0.0,
                            hold_secs: 0.1,
                            ramp_out_secs: 0.0,
                            easing: EaseFunction::Linear,
                            priority: 0,
                        })],
                    },
                ],
            },
        )]
        .into_iter()
        .collect(),
    });
    app.init_resource::<RecipeRuntime>();
    app.add_message::<AddTrauma>();
    app.add_message::<RequestCameraImpulse>();
    app.add_message::<RequestFlash>();
    app.add_message::<RequestHitstop>();
    app.add_message::<RequestTimeScale>();
    app.add_message::<RequestSquashStretch>();
    app.add_message::<PlayFeedbackRecipe>();
    app.add_message::<FeedbackStepFired>();
    app.add_systems(
        Update,
        (
            update_recipe_cooldowns,
            start_recipe_players,
            advance_recipe_players,
        )
            .chain(),
    );

    app.world_mut()
        .resource_mut::<Messages<PlayFeedbackRecipe>>()
        .write(PlayFeedbackRecipe::new("timed"));

    app.world_mut()
        .resource_mut::<GlobalTimeScale>()
        .unscaled_delta_secs = 0.05;
    app.update();
    let fired: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<FeedbackStepFired>>()
        .drain()
        .collect();
    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].step_name, "instant");
    assert!(
        app.world()
            .resource::<Messages<RequestTimeScale>>()
            .is_empty()
    );

    app.world_mut()
        .resource_mut::<GlobalTimeScale>()
        .unscaled_delta_secs = 0.10;
    app.update();
    assert!(
        app.world()
            .resource::<Messages<FeedbackStepFired>>()
            .is_empty(),
        "delayed step should not fire before 0.2 seconds"
    );

    app.world_mut()
        .resource_mut::<GlobalTimeScale>()
        .unscaled_delta_secs = 0.10;
    app.update();
    let fired: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<FeedbackStepFired>>()
        .drain()
        .collect();
    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].step_name, "delayed");
    assert!(
        !app.world()
            .resource::<Messages<RequestTimeScale>>()
            .is_empty()
    );
}
