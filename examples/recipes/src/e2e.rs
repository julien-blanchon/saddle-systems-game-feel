use bevy::prelude::*;
use saddle_bevy_e2e::{
    E2EPlugin, E2ESet, action::Action, actions::assertions, init_scenario, scenario::Scenario,
};
use saddle_systems_game_feel::{GameFeelDiagnostics, GameFeelSystems};
use saddle_systems_game_feel_example_support as support;

pub struct RecipesE2EPlugin;

impl Plugin for RecipesE2EPlugin {
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
                    "[game_feel_recipes:e2e] Unknown scenario '{name}'. Available: {:?}",
                    list_scenarios()
                );
            }
        }
    }
}

fn list_scenarios() -> Vec<&'static str> {
    vec!["game_feel_recipes_cycle"]
}

fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "game_feel_recipes_cycle" => Some(recipes_cycle()),
        _ => None,
    }
}

fn recipes_cycle() -> Scenario {
    Scenario::builder("game_feel_recipes_cycle")
        .description("Wait for the recipes example to trigger its automatic preset playback loop.")
        .then(Action::WaitUntil {
            label: "recipe player active".into(),
            condition: Box::new(|world| {
                world
                    .resource::<GameFeelDiagnostics>()
                    .active_recipe_players
                    > 0
            }),
            max_frames: 160,
        })
        .then(assertions::custom(
            "recipes example started a preset recipe",
            |world| world.resource::<crate::RecipeCycle>().0 > 0,
        ))
        .then(Action::Screenshot("recipes_cycle_peak".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitUntil {
            label: "recipe player cleared".into(),
            condition: Box::new(|world| {
                world
                    .resource::<GameFeelDiagnostics>()
                    .active_recipe_players
                    == 0
            }),
            max_frames: 180,
        })
        .then(Action::Screenshot("recipes_cycle_restored".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("game_feel_recipes_cycle summary"))
        .build()
}
