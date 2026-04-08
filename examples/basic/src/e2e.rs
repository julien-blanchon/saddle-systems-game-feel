use bevy::prelude::*;
use saddle_bevy_e2e::{
    E2EPlugin, E2ESet, action::Action, actions::assertions, init_scenario, scenario::Scenario,
};
use saddle_systems_game_feel::{GameFeelDiagnostics, GameFeelSystems};
use saddle_systems_game_feel_example_support as support;

pub struct BasicE2EPlugin;

impl Plugin for BasicE2EPlugin {
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
                    "[game_feel_basic:e2e] Unknown scenario '{name}'. Available: {:?}",
                    list_scenarios()
                );
            }
        }
    }
}

fn list_scenarios() -> Vec<&'static str> {
    vec!["game_feel_basic_loop"]
}

fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "game_feel_basic_loop" => Some(basic_loop()),
        _ => None,
    }
}

fn basic_loop() -> Scenario {
    Scenario::builder("game_feel_basic_loop")
        .description(
            "Wait for the self-running basic loop to trigger shake and punch on the demo camera.",
        )
        .then(Action::WaitUntil {
            label: "basic pulse active".into(),
            condition: Box::new(|world| {
                let diagnostics = world.resource::<GameFeelDiagnostics>();
                diagnostics.active_shake_listeners > 0 || diagnostics.active_punch_listeners > 0
            }),
            max_frames: 120,
        })
        .then(assertions::custom(
            "basic pulse activates feedback listeners",
            |world| {
                let diagnostics = world.resource::<GameFeelDiagnostics>();
                diagnostics.active_shake_listeners > 0 || diagnostics.active_punch_listeners > 0
            },
        ))
        .then(Action::Screenshot("basic_pulse".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("game_feel_basic_loop summary"))
        .build()
}
