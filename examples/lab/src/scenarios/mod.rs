use bevy::ecs::message::Messages;
use bevy::prelude::*;
use saddle_bevy_e2e::{
    action::Action, actions::assertions, scenario::Scenario, snapshot::Snapshot,
};
use saddle_systems_game_feel::{
    AddTrauma, FeedbackContext, ImpulseSpace, ListenerTarget, PlayFeedbackRecipe,
    RequestCameraImpulse, RequestFlash, RequestHitstop, RequestSquashStretch, RequestTimeScale,
    TimeScaleTarget, presets,
};

use crate::{LabControl, LabEvidence, LabMode, reset_lab};

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "smoke_launch",
        "shake_focus",
        "hitstop_flash",
        "recipe_showcase",
        "snap_restored_state",
        "combo_recipe",
        "time_scale_pulse",
        "recoil_3d_punch",
        "comparison_toggle",
        "debug_cycle_presets",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "smoke_launch" => Some(smoke_launch()),
        "shake_focus" => Some(shake_focus()),
        "hitstop_flash" => Some(hitstop_flash()),
        "recipe_showcase" => Some(recipe_showcase()),
        "snap_restored_state" => Some(snap_restored_state()),
        "combo_recipe" => Some(combo_recipe()),
        "time_scale_pulse" => Some(time_scale_pulse()),
        "recoil_3d_punch" => Some(recoil_3d_punch()),
        "comparison_toggle" => Some(comparison_toggle()),
        "debug_cycle_presets" => Some(debug_cycle_presets()),
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
        .then(assertions::resource_satisfies::<
            saddle_systems_game_feel::GlobalTimeScale,
        >("global time initialized", |time| {
            (time.scale - 1.0).abs() < 0.000_1
        }))
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
        .description("Cycle optional preset and repeating showcase recipes, assert step firing, hook cues, and rumble output, then capture the result.")
        .then(Action::Custom(Box::new(|world| reset_lab(world, LabMode::RecipeShowcase))))
        .then(Action::WaitUntil {
            label: "recipes fired".into(),
            condition: Box::new(|world| world.resource::<LabEvidence>().recipe_steps >= 6),
            max_frames: 220,
        })
        .then(assertions::resource_satisfies::<LabEvidence>(
            "recipe steps recorded",
            |evidence| evidence.recipe_steps >= 6,
        ))
        .then(assertions::resource_satisfies::<LabEvidence>(
            "recipe hook messages fired for audio or particle bridges",
            |evidence| evidence.hook_messages >= 4,
        ))
        .then(assertions::resource_satisfies::<LabEvidence>(
            "recipe visuals exercised",
            |evidence| {
                evidence.max_screen_flash > 0.08
                    && evidence.max_target_scale_delta > 0.1
                    && evidence.max_rumble > 0.1
            },
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

// ---------------------------------------------------------------------------
// New scenarios
// ---------------------------------------------------------------------------

/// Verifies the multi-step combo recipe loop: fires the lab's built-in
/// "combo_loop" recipe (two loops × two steps) and asserts that step firing
/// events, hook messages, and rumble output all land.  Mirrors what the
/// combo_system example exercises automatically.
fn combo_recipe() -> Scenario {
    Scenario::builder("combo_recipe")
        .description(
            "Play the combo_loop recipe twice and verify step events, hook messages, and rumble.",
        )
        .then(Action::Custom(Box::new(|world| {
            reset_lab(world, LabMode::Idle);

            let camera = world
                .query_filtered::<Entity, With<crate::support::DemoCamera>>()
                .iter(world)
                .next();
            let target = {
                let mut targets = world
                    .query_filtered::<(Entity, &Transform), With<crate::support::DemoTarget>>();
                targets
                    .iter(world)
                    .next()
                    .map(|(entity, transform)| (entity, transform.translation))
            };

            let (Some(camera), Some((target, origin))) = (camera, target) else {
                return;
            };

            world
                .resource_mut::<Messages<PlayFeedbackRecipe>>()
                .write(PlayFeedbackRecipe {
                    name: "combo_loop".into(),
                    context: FeedbackContext {
                        listener: Some(camera),
                        target: Some(target),
                        group: vec![target],
                        origin: Some(origin),
                        direction: Vec3::new(1.0, -0.2, 0.0),
                        channels: presets::channels::WEAPON,
                        ..Default::default()
                    },
                });
        })))
        .then(Action::WaitUntil {
            label: "combo recipe steps fired".into(),
            condition: Box::new(|world| world.resource::<LabEvidence>().recipe_steps >= 4),
            max_frames: 240,
        })
        .then(assertions::resource_satisfies::<LabEvidence>(
            "combo recipe steps recorded",
            |evidence| evidence.recipe_steps >= 4,
        ))
        .then(assertions::resource_satisfies::<LabEvidence>(
            "combo hook messages fired",
            |evidence| evidence.hook_messages >= 4,
        ))
        .then(Action::Screenshot("combo_recipe_peak".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitUntil {
            label: "combo recipe finished".into(),
            condition: Box::new(|world| {
                world
                    .resource::<saddle_systems_game_feel::GameFeelDiagnostics>()
                    .active_recipe_players
                    == 0
            }),
            max_frames: 240,
        })
        .then(assertions::resource_satisfies::<
            saddle_systems_game_feel::GameFeelDiagnostics,
        >("no recipe players remain", |diag| {
            diag.active_recipe_players == 0
        }))
        .then(assertions::resource_satisfies::<LabEvidence>(
            "rumble was produced during combo",
            |evidence| evidence.max_rumble > 0.0,
        ))
        .then(Action::Screenshot("combo_recipe_restored".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("combo_recipe summary"))
        .build()
}

/// Verifies the RequestTimeScale / global slow-motion path.  Injects a
/// slow-mo pulse directly into the message queue (mimicking what the
/// time_scale example does on a repeating timer) and asserts the
/// GlobalTimeScale resource drops below 0.5 then recovers to near 1.0.
fn time_scale_pulse() -> Scenario {
    Scenario::builder("time_scale_pulse")
        .description(
            "Inject a slow-mo RequestTimeScale, assert GlobalTimeScale drops then recovers.",
        )
        .then(Action::Custom(Box::new(|world| {
            reset_lab(world, LabMode::Idle);
        })))
        // Let the lab settle to idle.
        .then(Action::WaitFrames(10))
        // Fire a slow-mo pulse: ramp down fast, hold briefly, ramp back up.
        .then(Action::Custom(Box::new(|world| {
            world
                .resource_mut::<Messages<RequestTimeScale>>()
                .write(RequestTimeScale {
                    target: TimeScaleTarget::World,
                    scale: 0.10,
                    ramp_in_secs: 0.02,
                    hold_secs: 0.12,
                    ramp_out_secs: 0.22,
                    easing: bevy::math::curve::easing::EaseFunction::SineInOut,
                    priority: 0,
                    group_entities: Vec::new(),
                });
        })))
        // Wait until GlobalTimeScale clearly drops.
        .then(Action::WaitUntil {
            label: "time scale dropped".into(),
            condition: Box::new(|world| {
                world
                    .resource::<saddle_systems_game_feel::GlobalTimeScale>()
                    .scale
                    < 0.5
            }),
            max_frames: 60,
        })
        .then(assertions::resource_satisfies::<
            saddle_systems_game_feel::GlobalTimeScale,
        >("global scale dropped below 0.5", |t| {
            t.scale < 0.5
        }))
        .then(assertions::resource_satisfies::<LabEvidence>(
            "evidence recorded scale drop",
            |evidence| evidence.min_global_scale < 0.5,
        ))
        .then(Action::Screenshot("time_scale_low".into()))
        .then(Action::WaitFrames(1))
        // Wait for recovery back to near 1.0.
        .then(Action::WaitUntil {
            label: "time scale recovered".into(),
            condition: Box::new(|world| {
                world
                    .resource::<saddle_systems_game_feel::GlobalTimeScale>()
                    .scale
                    > 0.98
            }),
            max_frames: 120,
        })
        .then(assertions::resource_satisfies::<
            saddle_systems_game_feel::GlobalTimeScale,
        >("global scale restored to ~1.0", |t| {
            t.scale > 0.98
        }))
        .then(Action::Screenshot("time_scale_restored".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("time_scale_pulse summary"))
        .build()
}

/// Verifies the 3D recoil pattern: a local-space camera impulse (punch) plus
/// micro-trauma on a perspective camera, as demonstrated by the recoil_3d
/// example.  The lab uses a 2D camera so we test punch + trauma directly on
/// the lab's DemoCamera entity and confirm the listener counters fire.
fn recoil_3d_punch() -> Scenario {
    Scenario::builder("recoil_3d_punch")
        .description(
            "Fire a local-space camera impulse and micro trauma, verify punch/shake listeners.",
        )
        .then(Action::Custom(Box::new(|world| {
            reset_lab(world, LabMode::Idle);
        })))
        .then(Action::WaitFrames(10))
        // Inject a camera impulse (punch) + micro trauma targeting All listeners.
        .then(Action::Custom(Box::new(|world| {
            world
                .resource_mut::<Messages<AddTrauma>>()
                .write(AddTrauma {
                    target: ListenerTarget::All,
                    trauma: 0.12,
                    origin: None,
                    attenuation: None,
                    propagation_speed: None,
                    directional_bias: Vec3::new(0.02, -0.01, 0.0),
                    profile_override: None,
                });
            world
                .resource_mut::<Messages<RequestCameraImpulse>>()
                .write(RequestCameraImpulse {
                    target: ListenerTarget::All,
                    translation: Vec3::new(0.0, 0.06, 0.0),
                    rotation: Vec3::new(0.0, 0.0, -0.08),
                    fov: 0.03,
                    origin: None,
                    attenuation: None,
                    propagation_speed: None,
                    space: ImpulseSpace::Local,
                    profile_override: None,
                });
        })))
        // Wait for the punch listener to become active.
        .then(Action::WaitUntil {
            label: "punch listener active".into(),
            condition: Box::new(|world| {
                world
                    .resource::<saddle_systems_game_feel::GameFeelDiagnostics>()
                    .active_punch_listeners
                    > 0
                    || world
                        .resource::<saddle_systems_game_feel::GameFeelDiagnostics>()
                        .active_shake_listeners
                        > 0
            }),
            max_frames: 30,
        })
        .then(assertions::resource_satisfies::<
            saddle_systems_game_feel::GameFeelDiagnostics,
        >("punch or shake listener active", |diag| {
            diag.active_punch_listeners > 0 || diag.active_shake_listeners > 0
        }))
        .then(assertions::resource_satisfies::<LabEvidence>(
            "camera displaced by recoil",
            |evidence| {
                evidence.camera_distance_from_baseline > 0.005 || evidence.max_camera_shake > 0.005
            },
        ))
        .then(Action::Screenshot("recoil_peak".into()))
        .then(Action::WaitFrames(1))
        // Wait for spring to settle.
        .then(Action::WaitUntil {
            label: "recoil settled".into(),
            condition: Box::new(|world| {
                let evidence = world.resource::<LabEvidence>();
                let diag = world.resource::<saddle_systems_game_feel::GameFeelDiagnostics>();
                evidence.camera_distance_from_baseline < 0.02 && diag.active_punch_listeners == 0
            }),
            max_frames: 180,
        })
        .then(assertions::resource_satisfies::<LabEvidence>(
            "camera returned to baseline after recoil",
            |evidence| evidence.camera_distance_from_baseline < 0.02,
        ))
        .then(Action::Screenshot("recoil_settled".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("recoil_3d_punch summary"))
        .build()
}

/// Verifies the full "comparison" hit combo: trauma + camera impulse + entity
/// flash + screen pulse + hitstop + squash-stretch all fire simultaneously,
/// which is the same compound effect the comparison example fires every tick.
/// Asserts that every subsystem activates in the same frame window.
fn comparison_toggle() -> Scenario {
    Scenario::builder("comparison_toggle")
        .description("Fire all game-feel subsystems at once and assert simultaneous activation.")
        .then(Action::Custom(Box::new(|world| {
            reset_lab(world, LabMode::Idle);
        })))
        .then(Action::WaitFrames(10))
        // Fire the full compound hit effect that the comparison example uses.
        .then(Action::Custom(Box::new(|world| {
            // Resolve the camera and target entities for addressing.
            let camera = world
                .query_filtered::<Entity, With<crate::support::DemoCamera>>()
                .iter(world)
                .next();
            let target = world
                .query_filtered::<Entity, With<crate::support::DemoTarget>>()
                .iter(world)
                .next();

            let camera_target = match camera {
                Some(e) => ListenerTarget::Entity(e),
                None => ListenerTarget::All,
            };

            world
                .resource_mut::<Messages<AddTrauma>>()
                .write(AddTrauma {
                    target: camera_target,
                    trauma: 0.25,
                    origin: None,
                    attenuation: None,
                    propagation_speed: None,
                    directional_bias: Vec3::new(0.10, -0.04, 0.0),
                    profile_override: None,
                });

            world
                .resource_mut::<Messages<RequestCameraImpulse>>()
                .write(RequestCameraImpulse {
                    target: camera_target,
                    translation: Vec3::new(-0.12, 0.04, 0.0),
                    rotation: Vec3::new(0.0, 0.0, -0.08),
                    fov: 0.03,
                    origin: None,
                    attenuation: None,
                    propagation_speed: None,
                    space: ImpulseSpace::Local,
                    profile_override: None,
                });

            world
                .resource_mut::<Messages<RequestHitstop>>()
                .write(RequestHitstop {
                    target: TimeScaleTarget::World,
                    hold_frames: 4,
                    recovery_frames: 6,
                    ..RequestHitstop::new(TimeScaleTarget::World, 4)
                });

            if let Some(target_entity) = target {
                let camera_listener = camera
                    .map(ListenerTarget::Entity)
                    .unwrap_or(ListenerTarget::All);

                world
                    .resource_mut::<Messages<RequestFlash>>()
                    .write(RequestFlash {
                        target: saddle_systems_game_feel::FlashTarget::EntityAndScreen {
                            entity: target_entity,
                            screen: camera_listener,
                        },
                        color: Color::WHITE,
                        intensity: 0.45,
                        chromatic_aberration: 0.06,
                        vignette: 0.14,
                        origin: None,
                        attenuation: None,
                        duration_secs: 0.18,
                        easing: bevy::math::curve::easing::EaseFunction::SineOut,
                        clock: saddle_systems_game_feel::EffectTimeDomain::Unscaled,
                    });

                world
                    .resource_mut::<Messages<RequestSquashStretch>>()
                    .write(
                        RequestSquashStretch::new(target_entity, Vec3::new(1.20, 0.80, 1.0))
                            .with_duration(0.18),
                    );
            }
        })))
        // Wait for all subsystems to become active simultaneously.
        .then(Action::WaitUntil {
            label: "all subsystems active".into(),
            condition: Box::new(|world| {
                let diag = world.resource::<saddle_systems_game_feel::GameFeelDiagnostics>();
                let evidence = world.resource::<LabEvidence>();
                (diag.active_punch_listeners > 0 || diag.active_shake_listeners > 0)
                    && evidence.current_target_flash > 0.01
                    && evidence.current_screen_flash > 0.01
            }),
            max_frames: 30,
        })
        .then(assertions::resource_satisfies::<
            saddle_systems_game_feel::GameFeelDiagnostics,
        >("punch or shake active", |diag| {
            diag.active_punch_listeners > 0 || diag.active_shake_listeners > 0
        }))
        .then(assertions::resource_satisfies::<LabEvidence>(
            "entity flash active",
            |evidence| evidence.current_target_flash > 0.01,
        ))
        .then(assertions::resource_satisfies::<LabEvidence>(
            "screen flash active",
            |evidence| evidence.current_screen_flash > 0.01,
        ))
        .then(assertions::resource_satisfies::<LabEvidence>(
            "time scale dropped during hitstop",
            |evidence| evidence.min_global_scale < 0.5,
        ))
        .then(assertions::resource_satisfies::<LabEvidence>(
            "squash-stretch active on target",
            |evidence| evidence.current_target_scale_delta > 0.01,
        ))
        .then(Action::Screenshot("comparison_peak".into()))
        .then(Action::WaitFrames(1))
        // Wait for all outputs to clear.
        .then(Action::WaitUntil {
            label: "all effects cleared".into(),
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
            "all outputs cleared",
            |evidence| {
                evidence.current_target_flash < 0.01
                    && evidence.current_screen_flash < 0.01
                    && evidence.current_target_scale_delta < 0.02
                    && evidence.camera_distance_from_baseline < 0.02
            },
        ))
        .then(Action::Screenshot("comparison_cleared".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("comparison_toggle summary"))
        .build()
}

/// Verifies the preset recipe cycle used by the debug_showcase example:
/// cycles through HEAVY_IMPACT → EXPLOSION → REWARD_PING and asserts that
/// each recipe produces measurable effect output (flash, shake, steps).
fn debug_cycle_presets() -> Scenario {
    Scenario::builder("debug_cycle_presets")
        .description(
            "Play HEAVY_IMPACT, EXPLOSION, and REWARD_PING presets and verify each fires steps.",
        )
        // Start from a clean baseline.
        .then(Action::Custom(Box::new(|world| {
            reset_lab(world, LabMode::Idle);
        })))
        .then(Action::WaitFrames(10))
        // --- Round 1: HEAVY_IMPACT ---
        .then(Action::Custom(Box::new(|world| {
            *world.resource_mut::<LabEvidence>() = LabEvidence::default();
            let camera = world
                .query_filtered::<Entity, With<crate::support::DemoCamera>>()
                .iter(world)
                .next();
            let target = world
                .query_filtered::<Entity, With<crate::support::DemoTarget>>()
                .iter(world)
                .next();
            world
                .resource_mut::<Messages<PlayFeedbackRecipe>>()
                .write(PlayFeedbackRecipe {
                    name: presets::recipes::HEAVY_IMPACT.into(),
                    context: FeedbackContext {
                        listener: camera,
                        target,
                        group: target.map(|e| vec![e]).unwrap_or_default(),
                        origin: None,
                        direction: Vec3::new(1.0, -0.2, 0.0),
                        channels: presets::channels::WEAPON,
                        ..Default::default()
                    },
                });
        })))
        .then(Action::WaitUntil {
            label: "heavy_impact steps fired".into(),
            condition: Box::new(|world| {
                world.resource::<LabEvidence>().recipe_steps >= 1
                    || world.resource::<LabEvidence>().max_camera_shake > 0.005
            }),
            max_frames: 120,
        })
        .then(assertions::resource_satisfies::<LabEvidence>(
            "heavy_impact produced camera shake or steps",
            |evidence| evidence.max_camera_shake > 0.005 || evidence.recipe_steps >= 1,
        ))
        .then(Action::Screenshot("preset_heavy_impact".into()))
        .then(Action::WaitFrames(1))
        // Wait for recipe to finish before next round.
        .then(Action::WaitUntil {
            label: "heavy_impact recipe done".into(),
            condition: Box::new(|world| {
                world
                    .resource::<saddle_systems_game_feel::GameFeelDiagnostics>()
                    .active_recipe_players
                    == 0
            }),
            max_frames: 180,
        })
        // --- Round 2: EXPLOSION ---
        .then(Action::Custom(Box::new(|world| {
            *world.resource_mut::<LabEvidence>() = LabEvidence::default();
            let camera = world
                .query_filtered::<Entity, With<crate::support::DemoCamera>>()
                .iter(world)
                .next();
            let target = world
                .query_filtered::<Entity, With<crate::support::DemoTarget>>()
                .iter(world)
                .next();
            world
                .resource_mut::<Messages<PlayFeedbackRecipe>>()
                .write(PlayFeedbackRecipe {
                    name: presets::recipes::EXPLOSION.into(),
                    context: FeedbackContext {
                        listener: camera,
                        target,
                        group: target.map(|e| vec![e]).unwrap_or_default(),
                        origin: None,
                        direction: Vec3::new(-1.0, 0.1, 0.0),
                        channels: presets::channels::GAMEPLAY,
                        ..Default::default()
                    },
                });
        })))
        .then(Action::WaitUntil {
            label: "explosion steps fired".into(),
            condition: Box::new(|world| {
                world.resource::<LabEvidence>().recipe_steps >= 1
                    || world.resource::<LabEvidence>().max_camera_shake > 0.005
            }),
            max_frames: 120,
        })
        .then(assertions::resource_satisfies::<LabEvidence>(
            "explosion produced shake or screen flash",
            |evidence| evidence.max_camera_shake > 0.005 || evidence.max_screen_flash > 0.01,
        ))
        .then(Action::Screenshot("preset_explosion".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitUntil {
            label: "explosion recipe done".into(),
            condition: Box::new(|world| {
                world
                    .resource::<saddle_systems_game_feel::GameFeelDiagnostics>()
                    .active_recipe_players
                    == 0
            }),
            max_frames: 180,
        })
        // --- Round 3: REWARD_PING ---
        .then(Action::Custom(Box::new(|world| {
            *world.resource_mut::<LabEvidence>() = LabEvidence::default();
            let camera = world
                .query_filtered::<Entity, With<crate::support::DemoCamera>>()
                .iter(world)
                .next();
            let target = world
                .query_filtered::<Entity, With<crate::support::DemoTarget>>()
                .iter(world)
                .next();
            world
                .resource_mut::<Messages<PlayFeedbackRecipe>>()
                .write(PlayFeedbackRecipe {
                    name: presets::recipes::REWARD_PING.into(),
                    context: FeedbackContext {
                        listener: camera,
                        target,
                        group: target.map(|e| vec![e]).unwrap_or_default(),
                        origin: None,
                        direction: Vec3::Y,
                        channels: presets::channels::UI,
                        ..Default::default()
                    },
                });
        })))
        .then(Action::WaitUntil {
            label: "reward_ping steps or flash fired".into(),
            condition: Box::new(|world| {
                let evidence = world.resource::<LabEvidence>();
                evidence.recipe_steps >= 1
                    || evidence.max_screen_flash > 0.01
                    || evidence.max_target_flash > 0.01
            }),
            max_frames: 120,
        })
        .then(assertions::resource_satisfies::<LabEvidence>(
            "reward_ping produced flash or steps",
            |evidence| {
                evidence.recipe_steps >= 1
                    || evidence.max_screen_flash > 0.01
                    || evidence.max_target_flash > 0.01
            },
        ))
        .then(Action::Screenshot("preset_reward_ping".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitUntil {
            label: "reward_ping recipe done".into(),
            condition: Box::new(|world| {
                world
                    .resource::<saddle_systems_game_feel::GameFeelDiagnostics>()
                    .active_recipe_players
                    == 0
            }),
            max_frames: 120,
        })
        .then(assertions::log_summary("debug_cycle_presets summary"))
        .build()
}
