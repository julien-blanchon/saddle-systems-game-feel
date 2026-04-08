use bevy::prelude::*;
use saddle_bevy_e2e::{
    E2EPlugin, E2ESet, action::Action, actions::assertions, init_scenario, scenario::Scenario,
};
use saddle_systems_game_feel::{GameFeelDiagnostics, GameFeelSystems, PlayFeedbackRecipe, presets};
use saddle_systems_game_feel_example_support as support;

pub struct ShooterE2EPlugin;

impl Plugin for ShooterE2EPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(E2EPlugin);
        app.configure_sets(Update, E2ESet.before(GameFeelSystems::ProcessRequests));

        let args: Vec<String> = std::env::args().collect();
        let (scenario_name, handoff) = support::parse_e2e_args(&args);

        if let Some(name) = scenario_name {
            if let Some(mut scenario) = scenario_by_name(&name) {
                if handoff {
                    scenario.actions.push(Action::Handoff);
                }
                init_scenario(app, scenario);
            } else {
                error!(
                    "[game_feel_shooter:e2e] Unknown scenario '{name}'. Available: {:?}",
                    list_scenarios()
                );
            }
        }
    }
}

fn list_scenarios() -> Vec<&'static str> {
    vec!["game_feel_shooter_fire_and_hit"]
}

fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "game_feel_shooter_fire_and_hit" => Some(shooter_fire_and_hit()),
        _ => None,
    }
}

fn shooter_feedback_active(world: &World) -> bool {
    let diagnostics = world.resource::<GameFeelDiagnostics>();
    diagnostics.active_entity_flashes > 0
        || diagnostics.active_screen_pulses > 0
        || diagnostics.active_recipe_players > 0
        || diagnostics.active_punch_listeners > 0
        || diagnostics.active_shake_listeners > 0
}

fn queue_weapon_fire_from_example(world: &mut World) {
    let pane = world.resource::<support::ExampleFeelPane>().clone();
    let camera = world
        .query_filtered::<Entity, With<support::DemoCamera>>()
        .single(world)
        .expect("shooter example should spawn exactly one demo camera");
    let target = world
        .query_filtered::<Entity, With<crate::ShooterTarget>>()
        .iter(world)
        .next();

    let requests = crate::weapon_fire_requests(&pane, camera, target);
    let mut messages = world.resource_mut::<Messages<PlayFeedbackRecipe>>();
    for request in requests {
        messages.write(request);
    }
}

fn queue_gameplay_recipe_from_example(world: &mut World, recipe_name: &'static str) {
    let pane = world.resource::<support::ExampleFeelPane>().clone();
    let camera = world
        .query_filtered::<Entity, With<support::DemoCamera>>()
        .single(world)
        .expect("shooter example should spawn exactly one demo camera");
    let target = world
        .query_filtered::<Entity, With<crate::ShooterTarget>>()
        .iter(world)
        .next()
        .expect("shooter example should spawn at least one target");

    let request = crate::gameplay_recipe_request(recipe_name, &pane, camera, target);
    world
        .resource_mut::<Messages<PlayFeedbackRecipe>>()
        .write(request);
}

fn shooter_fire_and_hit() -> Scenario {
    Scenario::builder("game_feel_shooter_fire_and_hit")
        .description(
            "Trigger the shooter example's weapon-fire and heavy-impact cues through the same recipe wiring used by the live demo.",
        )
        .then(Action::WaitFrames(30))
        .then(Action::Custom(Box::new(queue_weapon_fire_from_example)))
        .then(Action::WaitUntil {
            label: "weapon fire active".into(),
            condition: Box::new(shooter_feedback_active),
            max_frames: 120,
        })
        .then(assertions::custom(
            "weapon fire produced visible feedback",
            shooter_feedback_active,
        ))
        .then(Action::Screenshot("shooter_weapon_fire".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitUntil {
            label: "weapon fire cleared".into(),
            condition: Box::new(|world| !shooter_feedback_active(world)),
            max_frames: 240,
        })
        .then(Action::Custom(Box::new(|world| {
            queue_gameplay_recipe_from_example(world, presets::recipes::HEAVY_IMPACT);
        })))
        .then(Action::WaitUntil {
            label: "heavy impact active".into(),
            condition: Box::new(shooter_feedback_active),
            max_frames: 120,
        })
        .then(assertions::custom(
            "heavy impact produced visible feedback",
            shooter_feedback_active,
        ))
        .then(Action::Screenshot("shooter_heavy_hit".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary(
            "game_feel_shooter_fire_and_hit summary",
        ))
        .build()
}
