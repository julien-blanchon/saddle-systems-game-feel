use bevy::prelude::*;
use saddle_bevy_e2e::{
    E2EPlugin, E2ESet, action::Action, actions::assertions, init_scenario, scenario::Scenario,
};
use saddle_systems_game_feel::{
    GameFeelSystems, PlayFeedbackRecipe, RequestCameraImpulse, RequestSquashStretch,
};
use saddle_systems_game_feel_example_support as support;

#[derive(Resource, Clone, Copy)]
struct PlatformerPlayerEntity(Entity);

pub struct PlatformerE2EPlugin;

impl Plugin for PlatformerE2EPlugin {
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
                    "[game_feel_platformer:e2e] Unknown scenario '{name}'. Available: {:?}",
                    list_scenarios()
                );
            }
        }
    }
}

fn list_scenarios() -> Vec<&'static str> {
    vec!["game_feel_platformer_jump_land"]
}

fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "game_feel_platformer_jump_land" => Some(platformer_jump_land()),
        _ => None,
    }
}

fn platformer_jump_land() -> Scenario {
    Scenario::builder("game_feel_platformer_jump_land")
        .description("Trigger a jump through the platformer example's jump logic, verify the player goes airborne, then lands back on the floor.")
        .then(Action::WaitFrames(30))
        .then(Action::Custom(Box::new(|world| {
            let player = world
                .query_filtered::<Entity, With<support::DemoTarget>>()
                .single(world)
                .expect("platformer example should spawn exactly one player");
            world.insert_resource(PlatformerPlayerEntity(player));
        })))
        .then(Action::Custom(Box::new(|world| {
            let player_entity = world.resource::<PlatformerPlayerEntity>().0;
            let pane = world.resource::<support::ExampleFeelPane>().clone();
            let camera = world
                .query_filtered::<Entity, With<support::DemoCamera>>()
                .single(world)
                .expect("platformer example should spawn exactly one demo camera");

            {
                let mut player = world
                    .get_mut::<crate::Player>(player_entity)
                    .expect("platformer player should exist");
                crate::apply_jump(&mut player);
            }

            let feedback = crate::build_jump_feedback(player_entity, &pane, camera);
            world
                .resource_mut::<Messages<RequestSquashStretch>>()
                .write(feedback.squash);
            world
                .resource_mut::<Messages<saddle_systems_game_feel::AddTrauma>>()
                .write(feedback.trauma);
            world
                .resource_mut::<Messages<RequestCameraImpulse>>()
                .write(feedback.impulse);
            world
                .resource_mut::<Messages<PlayFeedbackRecipe>>()
                .write(feedback.recipe);
        })))
        .then(Action::WaitUntil {
            label: "platformer player airborne".into(),
            condition: Box::new(|world| {
                let player = world.resource::<PlatformerPlayerEntity>().0;
                let player_state = world.get::<crate::Player>(player);
                let transform = world.get::<Transform>(player);
                match (player_state, transform) {
                    (Some(player_state), Some(transform)) => {
                        !player_state.grounded && transform.translation.y > crate::FLOOR_Y + 40.0
                    }
                    _ => false,
                }
            }),
            max_frames: 90,
        })
        .then(assertions::custom(
            "platformer jump leaves the ground",
            |world| {
                let player = world.resource::<PlatformerPlayerEntity>().0;
                let player_state = world.get::<crate::Player>(player);
                let transform = world.get::<Transform>(player);
                match (player_state, transform) {
                    (Some(player_state), Some(transform)) => {
                        !player_state.grounded && transform.translation.y > crate::FLOOR_Y + 40.0
                    }
                    _ => false,
                }
            },
        ))
        .then(Action::Screenshot("platformer_airborne".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitUntil {
            label: "platformer landed".into(),
            condition: Box::new(|world| {
                let player = world.resource::<PlatformerPlayerEntity>().0;
                let player_state = world.get::<crate::Player>(player);
                let transform = world.get::<Transform>(player);
                match (player_state, transform) {
                    (Some(player_state), Some(transform)) => {
                        player_state.grounded
                            && (transform.translation.y - (crate::FLOOR_Y + 32.0)).abs() < 0.1
                    }
                    _ => false,
                }
            }),
            max_frames: 180,
        })
        .then(Action::Screenshot("platformer_landed".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary(
            "game_feel_platformer_jump_land summary",
        ))
        .build()
}
