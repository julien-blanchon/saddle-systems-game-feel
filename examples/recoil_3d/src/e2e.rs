use bevy::prelude::*;
use saddle_bevy_e2e::{
    E2EPlugin, E2ESet, action::Action, actions::assertions, init_scenario, scenario::Scenario,
};
use saddle_systems_game_feel::{GameFeelSystems, PunchState, ShakeState};
use saddle_systems_game_feel_example_support as support;

#[derive(Resource, Clone, Copy)]
struct RecoilCameraEntity(Entity);

pub struct Recoil3dE2EPlugin;

impl Plugin for Recoil3dE2EPlugin {
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
                    "[game_feel_recoil_3d:e2e] Unknown scenario '{name}'. Available: {:?}",
                    list_scenarios()
                );
            }
        }
    }
}

fn list_scenarios() -> Vec<&'static str> {
    vec!["game_feel_recoil_3d_cycle"]
}

fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "game_feel_recoil_3d_cycle" => Some(recoil_3d_cycle()),
        _ => None,
    }
}

fn recoil_3d_cycle() -> Scenario {
    Scenario::builder("game_feel_recoil_3d_cycle")
        .description("Wait for the 3D recoil example to trigger local punch, shake, and additive FOV recoil.")
        .then(Action::Custom(Box::new(|world| {
            let camera = world
                .query_filtered::<Entity, With<crate::RecoilCamera>>()
                .single(world)
                .expect("3D recoil example should spawn exactly one recoil camera");
            world.insert_resource(RecoilCameraEntity(camera));
        })))
        .then(Action::WaitUntil {
            label: "3d recoil active".into(),
            condition: Box::new(|world| {
                let camera = world.resource::<RecoilCameraEntity>().0;
                let punch = world.get::<PunchState>(camera);
                let shake = world.get::<ShakeState>(camera);
                match (punch, shake) {
                    (Some(punch), shake) => {
                        punch.local_translation_offset.length() > 0.0001
                            || punch.rotation_offset.length() > 0.0001
                            || punch.fov_offset.abs() > 0.0001
                            || shake.is_some_and(|shake| shake.translation_offset.length() > 0.0001)
                    }
                    _ => false,
                }
            }),
            max_frames: 120,
        })
        .then(assertions::custom("3d recoil modifies camera outputs", |world| {
            let camera = world.resource::<RecoilCameraEntity>().0;
            world.get::<PunchState>(camera).is_some_and(|punch| {
                punch.local_translation_offset.length() > 0.0001
                    || punch.rotation_offset.length() > 0.0001
                    || punch.fov_offset.abs() > 0.0001
            })
        }))
        .then(Action::Screenshot("recoil_3d_peak".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("game_feel_recoil_3d_cycle summary"))
        .build()
}
