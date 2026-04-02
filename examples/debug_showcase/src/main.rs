use saddle_systems_game_feel_example_support as support;

use bevy::prelude::*;
use saddle_systems_game_feel::{
    FeedbackContext, GameFeelChannels, GameFeelPlugin, ListenerTarget, PlayFeedbackRecipe,
    RequestCameraImpulse,
};

#[derive(Resource)]
struct ShowcaseTimer(Timer);

#[derive(Resource, Default)]
struct ShowcaseCycle(u32);

fn main() {
    let mut app = App::new();
    support::add_example_plugins(
        &mut app,
        "Game Feel Debug Showcase",
        Color::srgb(0.04, 0.05, 0.07),
    );
    app.add_plugins(GameFeelPlugin::default());
    app.insert_resource(ShowcaseTimer(Timer::from_seconds(
        1.0,
        TimerMode::Repeating,
    )));
    app.insert_resource(ShowcaseCycle::default());
    app.insert_resource(support::HudLabel(
        "Debug Showcase\nAuto-cycles recoil, heavy impact, explosion, and reward pulses.".into(),
    ));
    app.add_systems(Startup, support::setup_2d_scene);
    app.add_systems(
        Update,
        (
            support::advance_demo_motion,
            support::update_hud,
            drive_showcase,
        ),
    );
    app.run();
}

fn drive_showcase(
    time: Res<Time>,
    mut timer: ResMut<ShowcaseTimer>,
    mut cycle: ResMut<ShowcaseCycle>,
    camera: Query<Entity, With<support::DemoCamera>>,
    target: Query<(Entity, &Transform), With<support::DemoTarget>>,
    mut recoil: MessageWriter<RequestCameraImpulse>,
    mut recipes: MessageWriter<PlayFeedbackRecipe>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let Ok(camera) = camera.single() else {
        return;
    };
    let Ok((target, transform)) = target.single() else {
        return;
    };

    match cycle.0 % 4 {
        0 => {
            recoil.write(RequestCameraImpulse {
                target: ListenerTarget::Entity(camera),
                translation: Vec3::new(-0.10, 0.04, 0.0),
                rotation: Vec3::new(0.0, 0.0, -0.08),
                fov: 0.0,
                origin: None,
                attenuation: None,
                propagation_speed: None,
                space: saddle_systems_game_feel::ImpulseSpace::Local,
                profile_override: None,
            });
        }
        1 => {
            recipes.write(PlayFeedbackRecipe {
                name: "heavy_impact".into(),
                context: FeedbackContext {
                    listener: Some(camera),
                    target: Some(target),
                    group: vec![target],
                    origin: Some(transform.translation),
                    direction: Vec3::new(1.0, -0.2, 0.0),
                    channels: GameFeelChannels::WEAPON,
                },
            });
        }
        2 => {
            recipes.write(PlayFeedbackRecipe {
                name: "explosion".into(),
                context: FeedbackContext {
                    listener: Some(camera),
                    target: Some(target),
                    group: vec![target],
                    origin: Some(transform.translation + Vec3::new(120.0, 0.0, 0.0)),
                    direction: Vec3::new(-1.0, 0.1, 0.0),
                    channels: GameFeelChannels::GAMEPLAY,
                },
            });
        }
        _ => {
            recipes.write(PlayFeedbackRecipe {
                name: "reward_ping".into(),
                context: FeedbackContext {
                    listener: Some(camera),
                    target: Some(target),
                    group: vec![target],
                    origin: Some(transform.translation),
                    direction: Vec3::Y,
                    channels: GameFeelChannels::UI,
                },
            });
        }
    };
    cycle.0 += 1;
}
