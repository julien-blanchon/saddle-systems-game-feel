use bevy::prelude::*;
use saddle_bevy_e2e::{
    E2EPlugin, E2ESet, action::Action, actions::assertions, init_scenario, scenario::Scenario,
};
use saddle_systems_game_feel::{GameFeelDiagnostics, GameFeelSystems, GlobalTimeScale};
use saddle_systems_game_feel_example_support as support;

pub struct HitstopE2EPlugin;

impl Plugin for HitstopE2EPlugin {
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
                    "[game_feel_hitstop:e2e] Unknown scenario '{name}'. Available: {:?}",
                    list_scenarios()
                );
            }
        }
    }
}

fn list_scenarios() -> Vec<&'static str> {
    vec!["game_feel_hitstop_cycle"]
}

fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "game_feel_hitstop_cycle" => Some(hitstop_cycle()),
        _ => None,
    }
}

fn hitstop_cycle() -> Scenario {
    Scenario::builder("game_feel_hitstop_cycle")
        .description("Wait for the automatic hitstop example to freeze time, flash the target, and recover cleanly.")
        .then(Action::WaitUntil {
            label: "hitstop and flash active".into(),
            condition: Box::new(|world| {
                let diagnostics = world.resource::<GameFeelDiagnostics>();
                let time = world.resource::<GlobalTimeScale>();
                time.scale < 0.95
                    && diagnostics.active_entity_flashes > 0
                    && diagnostics.active_screen_pulses > 0
            }),
            max_frames: 150,
        })
        .then(Action::Screenshot("hitstop_peak".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitUntil {
            label: "hitstop recovered".into(),
            condition: Box::new(|world| {
                let diagnostics = world.resource::<GameFeelDiagnostics>();
                let time = world.resource::<GlobalTimeScale>();
                time.scale > 0.99
                    && diagnostics.active_entity_flashes == 0
                    && diagnostics.active_screen_pulses == 0
            }),
            max_frames: 120,
        })
        .then(assertions::custom("hitstop example restored after the pulse", |world| {
            let diagnostics = world.resource::<GameFeelDiagnostics>();
            let time = world.resource::<GlobalTimeScale>();
            time.scale > 0.99
                && diagnostics.active_entity_flashes == 0
                && diagnostics.active_screen_pulses == 0
        }))
        .then(Action::Screenshot("hitstop_restored".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("game_feel_hitstop_cycle summary"))
        .build()
}
