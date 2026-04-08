use bevy::prelude::*;
use saddle_bevy_e2e::{
    E2EPlugin, E2ESet, action::Action, actions::assertions, init_scenario, scenario::Scenario,
};
use saddle_systems_game_feel::{
    GameFeelDiagnostics, GameFeelSystems, GameFeelToggles, GlobalTimeScale,
};
use saddle_systems_game_feel_example_support as support;

pub struct ComparisonE2EPlugin;

impl Plugin for ComparisonE2EPlugin {
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
                    "[game_feel_comparison:e2e] Unknown scenario '{name}'. Available: {:?}",
                    list_scenarios()
                );
            }
        }
    }
}

fn list_scenarios() -> Vec<&'static str> {
    vec!["game_feel_comparison_toggle"]
}

fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "game_feel_comparison_toggle" => Some(comparison_toggle()),
        _ => None,
    }
}

fn pulse_active(world: &World) -> bool {
    let diagnostics = world.resource::<GameFeelDiagnostics>();
    let time = world.resource::<GlobalTimeScale>();
    diagnostics.active_entity_flashes > 0
        || diagnostics.active_screen_pulses > 0
        || diagnostics.active_punch_listeners > 0
        || diagnostics.active_shake_listeners > 0
        || time.scale < 0.98
}

fn pulse_quiet(world: &World) -> bool {
    let diagnostics = world.resource::<GameFeelDiagnostics>();
    let time = world.resource::<GlobalTimeScale>();
    diagnostics.active_entity_flashes == 0
        && diagnostics.active_screen_pulses == 0
        && diagnostics.active_punch_listeners == 0
        && diagnostics.active_shake_listeners == 0
        && diagnostics.active_recipe_players == 0
        && time.scale > 0.99
}

fn comparison_toggle() -> Scenario {
    Scenario::builder("game_feel_comparison_toggle")
        .description("Wait for a full-feel pulse, disable effects with T, verify a quiet cycle, then re-enable them.")
        .then(Action::WaitUntil {
            label: "comparison pulse active".into(),
            condition: Box::new(pulse_active),
            max_frames: 150,
        })
        .then(Action::Screenshot("comparison_enabled".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world| {
            let mut toggles = world.resource_mut::<GameFeelToggles>();
            crate::set_effects_enabled(&mut toggles, false);
        })))
        .then(assertions::custom("comparison toggle disabled every effect family", |world| {
            let toggles = world.resource::<GameFeelToggles>();
            !toggles.shake_enabled
                && !toggles.punch_enabled
                && !toggles.flash_enabled
                && !toggles.hitstop_enabled
                && !toggles.time_scale_enabled
                && !toggles.rumble_enabled
                && !toggles.squash_stretch_enabled
                && !toggles.knockback_enabled
                && !toggles.screen_pulse_enabled
        }))
        .then(Action::WaitUntil {
            label: "comparison pulse cleared after toggle".into(),
            condition: Box::new(pulse_quiet),
            max_frames: 240,
        })
        .then(Action::WaitFrames(90))
        .then(assertions::custom("comparison toggle keeps the next cycle quiet", |world| {
            let toggles = world.resource::<GameFeelToggles>();
            pulse_quiet(world)
                && !toggles.shake_enabled
                && !toggles.punch_enabled
                && !toggles.flash_enabled
                && !toggles.hitstop_enabled
                && !toggles.time_scale_enabled
                && !toggles.rumble_enabled
                && !toggles.squash_stretch_enabled
                && !toggles.knockback_enabled
                && !toggles.screen_pulse_enabled
        }))
        .then(Action::Screenshot("comparison_disabled".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world| {
            let mut toggles = world.resource_mut::<GameFeelToggles>();
            crate::set_effects_enabled(&mut toggles, true);
        })))
        .then(assertions::custom("comparison toggle re-enabled every effect family", |world| {
            let toggles = world.resource::<GameFeelToggles>();
            toggles.shake_enabled
                && toggles.punch_enabled
                && toggles.flash_enabled
                && toggles.hitstop_enabled
                && toggles.time_scale_enabled
                && toggles.rumble_enabled
                && toggles.squash_stretch_enabled
                && toggles.knockback_enabled
                && toggles.screen_pulse_enabled
        }))
        .then(Action::WaitUntil {
            label: "comparison pulse active again".into(),
            condition: Box::new(pulse_active),
            max_frames: 150,
        })
        .then(assertions::custom("comparison toggle restores visible feedback", |world| {
            let toggles = world.resource::<GameFeelToggles>();
            let diagnostics = world.resource::<GameFeelDiagnostics>();
            toggles.shake_enabled
                && toggles.punch_enabled
                && toggles.flash_enabled
                && (diagnostics.active_entity_flashes > 0
                    || diagnostics.active_screen_pulses > 0
                    || diagnostics.active_punch_listeners > 0
                    || diagnostics.active_shake_listeners > 0)
        }))
        .then(Action::Screenshot("comparison_reenabled".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("game_feel_comparison_toggle summary"))
        .build()
}
