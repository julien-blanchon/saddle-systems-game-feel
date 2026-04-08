use bevy::prelude::*;
use saddle_bevy_e2e::{
    E2EPlugin, E2ESet, action::Action, actions::assertions, init_scenario, scenario::Scenario,
};
use saddle_systems_game_feel::{GameFeelConfig, GameFeelSystems, ScreenPulsePresentation};
use saddle_systems_game_feel_example_support as support;

pub struct ComboSystemE2EPlugin;

impl Plugin for ComboSystemE2EPlugin {
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
                    "[game_feel_combo_system:e2e] Unknown scenario '{name}'. Available: {:?}",
                    list_scenarios()
                );
            }
        }
    }
}

fn list_scenarios() -> Vec<&'static str> {
    vec!["game_feel_combo_system_cycle"]
}

fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "game_feel_combo_system_cycle" => Some(combo_system_cycle()),
        _ => None,
    }
}

fn combo_system_cycle() -> Scenario {
    Scenario::builder("game_feel_combo_system_cycle")
        .description("Let the combo example complete both loops and verify output-only screen pulses plus hook playback.")
        .then(Action::WaitUntil {
            label: "combo replay completed".into(),
            condition: Box::new(|world| {
                let state = world.resource::<crate::ComboHudState>();
                state.cycles_started > 0 && state.last_loop >= 2 && !state.last_hook.is_empty()
            }),
            max_frames: 320,
        })
        .then(assertions::custom("combo example uses output-only screen presentation", |world| {
            world.resource::<GameFeelConfig>().screen_presentation == ScreenPulsePresentation::OutputOnly
        }))
        .then(assertions::custom("combo example completed both loops", |world| {
            let state = world.resource::<crate::ComboHudState>();
            state.cycles_started > 0 && state.last_loop >= 2 && !state.last_step.is_empty()
        }))
        .then(Action::Screenshot("combo_system_cycle".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("game_feel_combo_system_cycle summary"))
        .build()
}
