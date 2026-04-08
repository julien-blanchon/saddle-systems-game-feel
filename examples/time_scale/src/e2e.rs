use bevy::prelude::*;
use saddle_bevy_e2e::{
    E2EPlugin, E2ESet, action::Action, actions::assertions, init_scenario, scenario::Scenario,
};
use saddle_systems_game_feel::{GameFeelSystems, GlobalTimeScale};
use saddle_systems_game_feel_example_support as support;

pub struct TimeScaleE2EPlugin;

impl Plugin for TimeScaleE2EPlugin {
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
                    "[game_feel_time_scale:e2e] Unknown scenario '{name}'. Available: {:?}",
                    list_scenarios()
                );
            }
        }
    }
}

fn list_scenarios() -> Vec<&'static str> {
    vec!["game_feel_time_scale_pulse"]
}

fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "game_feel_time_scale_pulse" => Some(time_scale_pulse()),
        _ => None,
    }
}

fn time_scale_pulse() -> Scenario {
    Scenario::builder("game_feel_time_scale_pulse")
        .description("Wait for the self-running time-scale example to enter and exit slow motion.")
        .then(Action::WaitUntil {
            label: "global slow motion active".into(),
            condition: Box::new(|world| world.resource::<GlobalTimeScale>().scale < 0.6),
            max_frames: 180,
        })
        .then(assertions::resource_satisfies::<GlobalTimeScale>(
            "global time scale dropped",
            |time| time.scale < 0.6,
        ))
        .then(Action::Screenshot("time_scale_low".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitUntil {
            label: "global slow motion recovered".into(),
            condition: Box::new(|world| world.resource::<GlobalTimeScale>().scale > 0.98),
            max_frames: 180,
        })
        .then(Action::Screenshot("time_scale_restored".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary(
            "game_feel_time_scale_pulse summary",
        ))
        .build()
}
