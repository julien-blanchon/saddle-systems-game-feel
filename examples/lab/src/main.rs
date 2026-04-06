#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;
use saddle_systems_game_feel_example_support as support;

use bevy::prelude::*;
#[cfg(feature = "dev")]
use bevy::remote::{RemotePlugin, http::RemoteHttpPlugin};
#[cfg(feature = "dev")]
use bevy_brp_extras::BrpExtrasPlugin;
use saddle_systems_game_feel::{
    AddTrauma, EffectTimeDomain, EntitySelector, FeedbackAction, FeedbackCondition,
    FeedbackContext, FeedbackHookTriggered, FeedbackRecipe, FeedbackRecipeLibrary,
    FeedbackRecipeRepeat, FeedbackStep, FeedbackStepFired, FlashOutput, FlashTarget,
    GameFeelDiagnostics, GameFeelPlugin, GlobalTimeScale, ListenerSelector, ListenerTarget,
    PlayFeedbackRecipe, RecipeHooks, RecipeRumble, RequestCameraImpulse, RequestFlash,
    RequestHitstop, RequestSquashStretch, RumbleOutput, ScreenPulseOutput, ShakeState,
    TimeScaleTarget, presets,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect)]
pub enum LabMode {
    Idle,
    ShakeBurst,
    HitstopFlash,
    RecipeShowcase,
}

#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct LabControl {
    pub mode: LabMode,
    pub mode_frame: u32,
    pub armed: bool,
    pub showcase_index: u32,
    pub showcase_remaining: u32,
}

impl Default for LabControl {
    fn default() -> Self {
        Self {
            mode: LabMode::Idle,
            mode_frame: 0,
            armed: false,
            showcase_index: 0,
            showcase_remaining: 0,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct LabBaselines {
    pub camera_translation: Vec3,
    pub target_translation: Vec3,
}

impl Default for LabBaselines {
    fn default() -> Self {
        Self {
            camera_translation: Vec3::ZERO,
            target_translation: Vec3::ZERO,
        }
    }
}

#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct LabEvidence {
    pub min_global_scale: f32,
    pub max_camera_shake: f32,
    pub max_target_flash: f32,
    pub max_screen_flash: f32,
    pub max_target_scale_delta: f32,
    pub current_target_flash: f32,
    pub current_screen_flash: f32,
    pub current_target_scale_delta: f32,
    pub max_rumble: f32,
    pub current_rumble: f32,
    pub recipe_steps: u32,
    pub hook_messages: u32,
    pub camera_distance_from_baseline: f32,
    pub target_distance_from_baseline: f32,
}

impl Default for LabEvidence {
    fn default() -> Self {
        Self {
            min_global_scale: 1.0,
            max_camera_shake: 0.0,
            max_target_flash: 0.0,
            max_screen_flash: 0.0,
            max_target_scale_delta: 0.0,
            current_target_flash: 0.0,
            current_screen_flash: 0.0,
            current_target_scale_delta: 0.0,
            max_rumble: 0.0,
            current_rumble: 0.0,
            recipe_steps: 0,
            hook_messages: 0,
            camera_distance_from_baseline: 0.0,
            target_distance_from_baseline: 0.0,
        }
    }
}

fn main() {
    let mut app = App::new();
    support::add_example_plugins(&mut app, "Game Feel Lab", Color::srgb(0.04, 0.05, 0.08));
    support::seed_example_pane(&mut app, support::ExampleFeelPane::default());
    #[cfg(feature = "dev")]
    app.add_plugins(RemotePlugin::default());
    #[cfg(feature = "dev")]
    app.add_plugins(BrpExtrasPlugin::with_http_plugin(
        RemoteHttpPlugin::default(),
    ));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::GameFeelLabE2EPlugin);

    app.register_type::<LabMode>();
    app.register_type::<LabControl>();
    app.register_type::<LabBaselines>();
    app.register_type::<LabEvidence>();
    app.insert_resource(LabControl::default());
    app.insert_resource(LabBaselines::default());
    app.insert_resource(LabEvidence::default());
    app.insert_resource(lab_recipe_library());
    app.add_plugins(GameFeelPlugin::default());
    app.add_systems(
        Startup,
        (support::setup_2d_scene, capture_baselines).chain(),
    );
    app.add_systems(
        Update,
        (
            support::advance_demo_motion,
            drive_lab_mode.before(saddle_systems_game_feel::GameFeelSystems::ProcessRequests),
            advance_mode_frame.after(drive_lab_mode),
        ),
    );
    app.add_systems(
        PostUpdate,
        (
            record_feedback_steps,
            track_lab_evidence,
            update_lab_overlay,
        )
            .chain()
            .after(saddle_systems_game_feel::GameFeelSystems::ApplyOutputs),
    );
    app.run();
}

pub fn reset_lab(world: &mut World, mode: LabMode) {
    *world.resource_mut::<LabControl>() = match mode {
        LabMode::RecipeShowcase => LabControl {
            mode,
            mode_frame: 0,
            armed: false,
            showcase_index: 0,
            showcase_remaining: 3,
        },
        _ => LabControl {
            mode,
            mode_frame: 0,
            armed: false,
            showcase_index: 0,
            showcase_remaining: 0,
        },
    };
    *world.resource_mut::<LabEvidence>() = LabEvidence::default();
}

fn lab_recipe_library() -> FeedbackRecipeLibrary {
    let mut library = presets::recipes::library();
    library.recipes.insert(
        "combo_loop".into(),
        FeedbackRecipe {
            cooldown_secs: 0.0,
            condition: FeedbackCondition::RequiresListener,
            repeat: FeedbackRecipeRepeat::Times {
                total_plays: 2,
                gap_secs: 0.22,
            },
            steps: vec![
                FeedbackStep {
                    name: "jab".into(),
                    at_secs: 0.0,
                    actions: vec![
                        FeedbackAction::Hooks(RecipeHooks {
                            audio_cue: Some("jab_whoosh".into()),
                            particle_cue: Some("small_hit_sparks".into()),
                            target: Some(EntitySelector::ContextTarget),
                            use_context_origin: true,
                        }),
                        FeedbackAction::Rumble(RecipeRumble {
                            target: ListenerSelector::ContextListener,
                            low_frequency: 0.24,
                            high_frequency: 0.42,
                            duration_secs: 0.12,
                            easing: bevy::math::curve::easing::EaseFunction::QuadraticOut,
                            clock: EffectTimeDomain::GlobalScaled,
                        }),
                    ],
                },
                FeedbackStep {
                    name: "finisher".into(),
                    at_secs: 0.18,
                    actions: vec![
                        FeedbackAction::Hooks(RecipeHooks {
                            audio_cue: Some("combo_finisher".into()),
                            particle_cue: Some("radial_ring".into()),
                            target: Some(EntitySelector::ContextTarget),
                            use_context_origin: true,
                        }),
                        FeedbackAction::Rumble(RecipeRumble {
                            target: ListenerSelector::ContextListener,
                            low_frequency: 0.62,
                            high_frequency: 0.36,
                            duration_secs: 0.2,
                            easing: bevy::math::curve::easing::EaseFunction::CubicOut,
                            clock: EffectTimeDomain::GlobalScaled,
                        }),
                    ],
                },
            ],
        },
    );
    library
}

fn capture_baselines(
    camera: Query<&Transform, With<support::DemoCamera>>,
    target: Query<&Transform, With<support::DemoTarget>>,
    mut baselines: ResMut<LabBaselines>,
) {
    if let Ok(camera) = camera.single() {
        baselines.camera_translation = camera.translation;
    }
    if let Ok(target) = target.single() {
        baselines.target_translation = target.translation;
    }
}

fn drive_lab_mode(
    mut control: ResMut<LabControl>,
    camera: Query<Entity, With<support::DemoCamera>>,
    target: Query<(Entity, &Transform), With<support::DemoTarget>>,
    mut trauma: MessageWriter<AddTrauma>,
    mut impulse: MessageWriter<RequestCameraImpulse>,
    mut flash: MessageWriter<RequestFlash>,
    mut hitstop: MessageWriter<RequestHitstop>,
    mut squash: MessageWriter<RequestSquashStretch>,
    mut recipes: MessageWriter<PlayFeedbackRecipe>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };
    let Ok((target, target_transform)) = target.single() else {
        return;
    };

    match control.mode {
        LabMode::Idle => {}
        LabMode::ShakeBurst if !control.armed => {
            trauma.write(AddTrauma {
                target: ListenerTarget::Entity(camera),
                trauma: 0.55,
                origin: None,
                attenuation: None,
                propagation_speed: None,
                directional_bias: Vec3::new(0.18, -0.05, 0.0),
                profile_override: None,
            });
            impulse.write(RequestCameraImpulse {
                target: ListenerTarget::Entity(camera),
                translation: Vec3::new(-0.22, 0.08, 0.0),
                rotation: Vec3::new(0.0, 0.0, -0.14),
                fov: 0.0,
                origin: None,
                attenuation: None,
                propagation_speed: None,
                space: saddle_systems_game_feel::ImpulseSpace::Local,
                profile_override: None,
            });
            control.armed = true;
        }
        LabMode::HitstopFlash if !control.armed => {
            trauma.write(AddTrauma {
                target: ListenerTarget::Entity(camera),
                trauma: 0.42,
                origin: Some(target_transform.translation),
                attenuation: None,
                propagation_speed: None,
                directional_bias: Vec3::new(0.16, -0.04, 0.0),
                profile_override: None,
            });
            hitstop.write(RequestHitstop {
                target: TimeScaleTarget::World,
                hold_frames: 4,
                recovery_frames: 6,
                ..RequestHitstop::new(TimeScaleTarget::World, 4)
            });
            flash.write(RequestFlash {
                target: FlashTarget::EntityAndScreen {
                    entity: target,
                    screen: ListenerTarget::Entity(camera),
                },
                color: Color::WHITE,
                intensity: 0.55,
                chromatic_aberration: 0.06,
                vignette: 0.14,
                origin: Some(target_transform.translation),
                attenuation: None,
                duration_secs: 0.18,
                easing: bevy::math::curve::easing::EaseFunction::SineOut,
                clock: saddle_systems_game_feel::EffectTimeDomain::Unscaled,
            });
            squash.write(RequestSquashStretch::new(
                target,
                Vec3::new(1.22, 0.78, 1.0),
            ));
            control.armed = true;
        }
        LabMode::RecipeShowcase
            if control.showcase_remaining > 0
                && control.mode_frame >= control.showcase_index.saturating_mul(48) =>
        {
            let recipe_name = match control.showcase_index % 3 {
                0 => presets::recipes::HEAVY_IMPACT,
                1 => "combo_loop",
                _ => presets::recipes::REWARD_PING,
            };
            recipes.write(PlayFeedbackRecipe {
                name: recipe_name.into(),
                context: FeedbackContext {
                    listener: Some(camera),
                    target: Some(target),
                    group: vec![target],
                    origin: Some(target_transform.translation),
                    direction: Vec3::new(1.0, -0.2, 0.0),
                    channels: presets::channels::WEAPON,
                    ..default()
                },
            });
            control.showcase_index += 1;
            control.showcase_remaining -= 1;
        }
        _ => {}
    }
}

fn record_feedback_steps(
    mut reader: MessageReader<FeedbackStepFired>,
    mut hook_reader: MessageReader<FeedbackHookTriggered>,
    mut evidence: ResMut<LabEvidence>,
) {
    evidence.recipe_steps += reader.read().count() as u32;
    evidence.hook_messages += hook_reader.read().count() as u32;
}

fn track_lab_evidence(
    baselines: Res<LabBaselines>,
    global: Res<GlobalTimeScale>,
    camera_transform: Query<&Transform, With<support::DemoCamera>>,
    target_transform: Query<&Transform, With<support::DemoTarget>>,
    camera_shake: Query<&ShakeState, With<support::DemoCamera>>,
    target_flash: Query<&FlashOutput, With<support::DemoTarget>>,
    screen_flash: Query<&ScreenPulseOutput, With<support::DemoCamera>>,
    rumble: Query<&RumbleOutput, With<support::DemoCamera>>,
    target_scale: Query<&saddle_systems_game_feel::SquashStretchState, With<support::DemoTarget>>,
    mut evidence: ResMut<LabEvidence>,
) {
    evidence.min_global_scale = evidence.min_global_scale.min(global.scale);

    if let Ok(shake) = camera_shake.single() {
        evidence.max_camera_shake = evidence
            .max_camera_shake
            .max(shake.translation_offset.length());
    }
    if let Ok(flash) = target_flash.single() {
        evidence.max_target_flash = evidence.max_target_flash.max(flash.intensity);
        evidence.current_target_flash = flash.intensity;
    } else {
        evidence.current_target_flash = 0.0;
    }
    if let Ok(pulse) = screen_flash.single() {
        evidence.max_screen_flash = evidence.max_screen_flash.max(pulse.flash_alpha);
        evidence.current_screen_flash = pulse.flash_alpha;
    } else {
        evidence.current_screen_flash = 0.0;
    }
    if let Ok(rumble) = rumble.single() {
        let current_rumble = rumble.low_frequency.max(rumble.high_frequency);
        evidence.max_rumble = evidence.max_rumble.max(current_rumble);
        evidence.current_rumble = current_rumble;
    } else {
        evidence.current_rumble = 0.0;
    }
    if let Ok(scale) = target_scale.single() {
        let current_scale_delta = (scale.scale_multiplier - Vec3::ONE).length();
        evidence.max_target_scale_delta = evidence.max_target_scale_delta.max(current_scale_delta);
        evidence.current_target_scale_delta = current_scale_delta;
    } else {
        evidence.current_target_scale_delta = 0.0;
    }
    if let Ok(transform) = camera_transform.single() {
        evidence.camera_distance_from_baseline =
            transform.translation.distance(baselines.camera_translation);
    }
    if let Ok(transform) = target_transform.single() {
        evidence.target_distance_from_baseline =
            transform.translation.distance(baselines.target_translation);
    }
}

fn update_lab_overlay(
    control: Res<LabControl>,
    global: Res<GlobalTimeScale>,
    diagnostics: Res<GameFeelDiagnostics>,
    evidence: Res<LabEvidence>,
    mut query: Query<&mut Text, With<support::DemoHud>>,
) {
    let Ok(mut text) = query.single_mut() else {
        return;
    };

    text.0 = format!(
        "Game Feel Lab\nMode {:?} frame {}\nGlobal scale {:.2}\nPeak shake {:.2}  target flash {:.2}/{:.2}  screen flash {:.2}/{:.2}  rumble {:.2}/{:.2}\nPeak squash {:.2}/{:.2}  recipe steps {}  hooks {}\nCamera drift {:.2}  target drift {:.2}\nDiagnostics: shake {} punch {} flash {} screen {} rumble {} recipes {}",
        control.mode,
        control.mode_frame,
        global.scale,
        evidence.max_camera_shake,
        evidence.current_target_flash,
        evidence.max_target_flash,
        evidence.current_screen_flash,
        evidence.max_screen_flash,
        evidence.current_rumble,
        evidence.max_rumble,
        evidence.current_target_scale_delta,
        evidence.max_target_scale_delta,
        evidence.recipe_steps,
        evidence.hook_messages,
        evidence.camera_distance_from_baseline,
        evidence.target_distance_from_baseline,
        diagnostics.active_shake_listeners,
        diagnostics.active_punch_listeners,
        diagnostics.active_entity_flashes,
        diagnostics.active_screen_pulses,
        diagnostics.active_rumble_listeners,
        diagnostics.active_recipe_players,
    );
}

fn advance_mode_frame(mut control: ResMut<LabControl>) {
    control.mode_frame = control.mode_frame.saturating_add(1);
}
