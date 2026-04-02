use saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario, snapshot::Snapshot};

use crate::{LabControl, LabEvidence, LabMode, reset_lab};

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "smoke_launch",
        "shake_focus",
        "hitstop_flash",
        "recipe_showcase",
        "snap_restored_state",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "smoke_launch" => Some(smoke_launch()),
        "shake_focus" => Some(shake_focus()),
        "hitstop_flash" => Some(hitstop_flash()),
        "recipe_showcase" => Some(recipe_showcase()),
        "snap_restored_state" => Some(snap_restored_state()),
        _ => None,
    }
}

fn smoke_launch() -> Scenario {
    Scenario::builder("smoke_launch")
        .description(
            "Boot the lab, verify the shared crate resources exist, and capture the idle baseline.",
        )
        .then(Action::WaitFrames(30))
        .then(assertions::entity_exists::<crate::support::DemoCamera>(
            "camera exists",
        ))
        .then(assertions::entity_exists::<crate::support::DemoTarget>(
            "target exists",
        ))
        .then(
            assertions::resource_satisfies::<saddle_systems_game_feel::GlobalTimeScale>(
                "global time initialized",
                |time| (time.scale - 1.0).abs() < 0.000_1,
            ),
        )
        .then(assertions::resource_satisfies::<LabEvidence>(
            "idle has no active feel output",
            |evidence| {
                evidence.max_camera_shake < 0.001
                    && evidence.min_global_scale > 0.99
                    && evidence.current_target_flash < 0.001
                    && evidence.current_screen_flash < 0.001
            },
        ))
        .then(Action::Screenshot("smoke_launch".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("smoke_launch summary"))
        .build()
}

fn shake_focus() -> Scenario {
    Scenario::builder("shake_focus")
        .description("Trigger a single shake burst, verify the camera moved, and capture peak plus recovered frames.")
        .then(Action::Custom(Box::new(|world| reset_lab(world, LabMode::ShakeBurst))))
        .then(Action::WaitUntil {
            label: "shake listeners active".into(),
            condition: Box::new(|world| {
                world
                    .resource::<saddle_systems_game_feel::GameFeelDiagnostics>()
                    .active_shake_listeners
                    > 0
            }),
            max_frames: 30,
        })
        .then(assertions::resource_satisfies::<saddle_systems_game_feel::GameFeelDiagnostics>(
            "camera shake listeners active",
            |diagnostics| diagnostics.active_shake_listeners > 0,
        ))
        .then(assertions::resource_satisfies::<LabEvidence>(
            "camera shake displaced the view",
            |evidence| {
                evidence.camera_distance_from_baseline > 0.01
                    || evidence.max_camera_shake > 0.01
            },
        ))
        .then(Action::Screenshot("shake_peak".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitUntil {
            label: "camera restored".into(),
            condition: Box::new(|world| {
                let evidence = world.resource::<LabEvidence>();
                evidence.camera_distance_from_baseline < 0.02
            }),
            max_frames: 180,
        })
        .then(assertions::resource_satisfies::<LabEvidence>(
            "camera returned to baseline",
            |evidence| evidence.camera_distance_from_baseline < 0.02,
        ))
        .then(Action::Screenshot("shake_restored".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("shake_focus summary"))
        .build()
}

fn hitstop_flash() -> Scenario {
    Scenario::builder("hitstop_flash")
        .description("Trigger hitstop plus flash, assert world scale dropped and visuals fired, then confirm recovery.")
        .then(Action::Custom(Box::new(|world| reset_lab(world, LabMode::HitstopFlash))))
        .then(Action::WaitUntil {
            label: "hitstop engaged".into(),
            condition: Box::new(|world| {
                let evidence = world.resource::<LabEvidence>();
                let diagnostics = world.resource::<saddle_systems_game_feel::GameFeelDiagnostics>();
                evidence.min_global_scale < 0.1
                    && diagnostics.active_entity_flashes > 0
                    && diagnostics.active_screen_pulses > 0
            }),
            max_frames: 30,
        })
        .then(assertions::custom(
            "world froze and target flashed",
            |world| {
                let evidence = world.resource::<LabEvidence>();
                let diagnostics = world.resource::<saddle_systems_game_feel::GameFeelDiagnostics>();
                evidence.min_global_scale < 0.1
                    && diagnostics.active_entity_flashes > 0
                    && diagnostics.active_screen_pulses > 0
            },
        ))
        .then(Action::Screenshot("hitstop_flash_peak".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitUntil {
            label: "effect recovered".into(),
            condition: Box::new(|world| {
                let evidence = world.resource::<LabEvidence>();
                let time = world.resource::<saddle_systems_game_feel::GlobalTimeScale>();
                time.scale > 0.99
                    && evidence.current_target_flash < 0.01
                    && evidence.current_screen_flash < 0.01
                    && evidence.current_target_scale_delta < 0.02
                    && evidence.camera_distance_from_baseline < 0.02
            }),
            max_frames: 180,
        })
        .then(assertions::resource_satisfies::<LabEvidence>(
            "active outputs cleared",
            |evidence| {
                evidence.current_target_flash < 0.01
                    && evidence.current_screen_flash < 0.01
                    && evidence.current_target_scale_delta < 0.02
                    && evidence.camera_distance_from_baseline < 0.02
            },
        ))
        .then(Action::Screenshot("hitstop_flash_restored".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("hitstop_flash summary"))
        .build()
}

fn recipe_showcase() -> Scenario {
    Scenario::builder("recipe_showcase")
        .description("Cycle the built-in showcase recipes, assert step firing and visual pulses, then capture the result.")
        .then(Action::Custom(Box::new(|world| reset_lab(world, LabMode::RecipeShowcase))))
        .then(Action::WaitUntil {
            label: "recipes fired".into(),
            condition: Box::new(|world| world.resource::<LabEvidence>().recipe_steps >= 5),
            max_frames: 220,
        })
        .then(assertions::resource_satisfies::<LabEvidence>(
            "recipe steps recorded",
            |evidence| evidence.recipe_steps >= 5,
        ))
        .then(assertions::resource_satisfies::<LabEvidence>(
            "recipe visuals exercised",
            |evidence| evidence.max_screen_flash > 0.08 && evidence.max_target_scale_delta > 0.1,
        ))
        .then(Action::Screenshot("recipe_showcase_peak".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitUntil {
            label: "recipe outputs restored".into(),
            condition: Box::new(|world| {
                let evidence = world.resource::<LabEvidence>();
                let diagnostics = world.resource::<saddle_systems_game_feel::GameFeelDiagnostics>();
                let time = world.resource::<saddle_systems_game_feel::GlobalTimeScale>();
                diagnostics.active_recipe_players == 0
                    && time.scale > 0.99
                    && evidence.current_screen_flash < 0.01
                    && evidence.current_target_scale_delta < 0.02
                    && evidence.camera_distance_from_baseline < 0.02
            }),
            max_frames: 240,
        })
        .then(assertions::custom(
            "recipe showcase returned to baseline",
            |world| {
                let evidence = world.resource::<LabEvidence>();
                let diagnostics = world.resource::<saddle_systems_game_feel::GameFeelDiagnostics>();
                let time = world.resource::<saddle_systems_game_feel::GlobalTimeScale>();
                diagnostics.active_recipe_players == 0
                    && time.scale > 0.99
                    && evidence.current_screen_flash < 0.01
                    && evidence.current_target_scale_delta < 0.02
                    && evidence.camera_distance_from_baseline < 0.02
            },
        ))
        .then(Action::Screenshot("recipe_showcase_restored".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("recipe_showcase summary"))
        .build()
}

fn snap_restored_state() -> Scenario {
    Snapshot::builder("snap_restored_state")
        .description("Reset the lab to idle and capture the fully recovered baseline state.")
        .setup(|world| reset_lab(world, LabMode::Idle))
        .settle(30)
        .action(assertions::resource_satisfies::<LabControl>(
            "lab returned to idle",
            |control| control.mode == LabMode::Idle,
        ))
        .action(assertions::resource_satisfies::<LabEvidence>(
            "idle state has no active feel output",
            |evidence| {
                evidence.camera_distance_from_baseline < 0.02
                    && evidence.current_target_flash < 0.001
                    && evidence.current_screen_flash < 0.001
                    && evidence.current_target_scale_delta < 0.001
            },
        ))
        .capture("restored_idle")
        .action(assertions::log_summary("snap_restored_state summary"))
        .build()
        .into_scenario()
}
