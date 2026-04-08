use bevy::prelude::*;
use saddle_bevy_e2e::{
    E2EPlugin, E2ESet, action::Action, actions::assertions, init_scenario, scenario::Scenario,
};
use saddle_systems_game_feel::{GameFeelDiagnostics, GameFeelSystems, KnockbackState};
use saddle_systems_game_feel_example_support as support;

#[derive(Resource, Clone, Copy)]
struct PunchBagEntity(Entity);

pub struct PunchingE2EPlugin;

impl Plugin for PunchingE2EPlugin {
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
                    "[game_feel_punching:e2e] Unknown scenario '{name}'. Available: {:?}",
                    list_scenarios()
                );
            }
        }
    }
}

fn list_scenarios() -> Vec<&'static str> {
    vec!["game_feel_punching_strike"]
}

fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "game_feel_punching_strike" => Some(punching_strike()),
        _ => None,
    }
}

fn punching_strike() -> Scenario {
    Scenario::builder("game_feel_punching_strike")
        .description("Press Space to punch the bag and verify knockback plus the rest of the stacked feedback path.")
        .then(Action::WaitFrames(30))
        .then(Action::Custom(Box::new(|world| {
            let bag = world
                .query_filtered::<Entity, With<crate::PunchBag>>()
                .single(world)
                .expect("punching example should spawn exactly one punch bag");
            world.insert_resource(PunchBagEntity(bag));
        })))
        .then(Action::PressKey(KeyCode::Space))
        .then(Action::WaitFrames(1))
        .then(Action::ReleaseKey(KeyCode::Space))
        .then(Action::WaitUntil {
            label: "punch bag displaced".into(),
            condition: Box::new(|world| {
                let bag = world.resource::<PunchBagEntity>().0;
                let diagnostics = world.resource::<GameFeelDiagnostics>();
                world
                    .get::<KnockbackState>(bag)
                    .is_some_and(|state| state.displacement.length() > 0.1)
                    && diagnostics.active_entity_flashes > 0
            }),
            max_frames: 90,
        })
        .then(assertions::custom("punching example applied knockback", |world| {
            let bag = world.resource::<PunchBagEntity>().0;
            world
                .get::<KnockbackState>(bag)
                .is_some_and(|state| state.displacement.length() > 0.1)
        }))
        .then(Action::Screenshot("punching_peak".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitUntil {
            label: "punch bag restored".into(),
            condition: Box::new(|world| {
                let bag = world.resource::<PunchBagEntity>().0;
                world
                    .get::<KnockbackState>(bag)
                    .is_some_and(|state| state.displacement.length() < 0.05)
            }),
            max_frames: 180,
        })
        .then(Action::Screenshot("punching_restored".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("game_feel_punching_strike summary"))
        .build()
}
