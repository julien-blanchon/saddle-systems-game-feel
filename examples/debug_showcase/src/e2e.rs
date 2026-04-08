use bevy::prelude::*;
use saddle_bevy_e2e::{
    E2EPlugin, E2ESet, action::Action, actions::assertions, init_scenario, scenario::Scenario,
};
use saddle_systems_game_feel::GameFeelSystems;
use saddle_systems_game_feel_example_support as support;

pub struct DebugShowcaseE2EPlugin;

impl Plugin for DebugShowcaseE2EPlugin {
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
                    "[game_feel_debug_showcase:e2e] Unknown scenario '{name}'. Available: {:?}",
                    list_scenarios()
                );
            }
        }
    }
}

fn list_scenarios() -> Vec<&'static str> {
    vec!["game_feel_debug_showcase_cycle"]
}

fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "game_feel_debug_showcase_cycle" => Some(debug_showcase_cycle()),
        _ => None,
    }
}

fn debug_showcase_cycle() -> Scenario {
    Scenario::builder("game_feel_debug_showcase_cycle")
        .description("Let the debug showcase complete a full four-step rotation of recoil and preset recipes.")
        .then(Action::WaitUntil {
            label: "debug showcase completed one full rotation".into(),
            condition: Box::new(|world| {
                world.resource::<crate::ShowcaseCycle>().0 >= 4
            }),
            max_frames: 360,
        })
        .then(assertions::custom("debug showcase reached all four auto steps", |world| {
            world.resource::<crate::ShowcaseCycle>().0 >= 4
        }))
        .then(Action::Screenshot("debug_showcase_cycle".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("game_feel_debug_showcase_cycle summary"))
        .build()
}
