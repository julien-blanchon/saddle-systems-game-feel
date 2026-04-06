use saddle_systems_game_feel_example_support as support;

use bevy::prelude::*;
use saddle_systems_game_feel::{FeedbackContext, GameFeelPlugin, PlayFeedbackRecipe, presets};

#[derive(Resource)]
struct RecipeTimer(Timer);

#[derive(Resource, Default)]
struct RecipeCycle(u32);

fn main() {
    let mut app = App::new();
    support::add_example_plugins(&mut app, "Game Feel Recipes", Color::srgb(0.05, 0.06, 0.09));
    support::seed_example_pane(
        &mut app,
        support::ExampleFeelPane {
            interval_secs: 1.35,
            ..default()
        },
    );
    app.add_plugins(GameFeelPlugin::default());
    app.insert_resource(presets::recipes::library());
    app.insert_resource(RecipeTimer(Timer::from_seconds(1.35, TimerMode::Repeating)));
    app.insert_resource(RecipeCycle::default());
    app.insert_resource(support::HudLabel(
        "Recipes\nAlternates optional heavy_impact, explosion, and reward_ping preset recipes."
            .into(),
    ));
    app.add_systems(Startup, support::setup_2d_scene);
    app.add_systems(
        Update,
        (
            support::advance_demo_motion,
            support::update_hud,
            play_recipe_cycle,
        ),
    );
    app.run();
}

fn play_recipe_cycle(
    time: Res<Time>,
    pane: Res<support::ExampleFeelPane>,
    mut timer: ResMut<RecipeTimer>,
    mut cycle: ResMut<RecipeCycle>,
    camera: Query<(Entity, &Transform), With<support::DemoCamera>>,
    target: Query<(Entity, &Transform), With<support::DemoTarget>>,
    mut recipes: MessageWriter<PlayFeedbackRecipe>,
) {
    timer.0.set_duration(std::time::Duration::from_secs_f32(
        pane.interval_secs.max(0.2),
    ));
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let Ok((camera, _)) = camera.single() else {
        return;
    };
    let Ok((target, transform)) = target.single() else {
        return;
    };

    let name = match cycle.0 % 3 {
        0 => presets::recipes::HEAVY_IMPACT,
        1 => presets::recipes::EXPLOSION,
        _ => presets::recipes::REWARD_PING,
    };
    cycle.0 += 1;

    recipes.write(PlayFeedbackRecipe {
        name: name.into(),
        context: FeedbackContext {
            listener: Some(camera),
            target: Some(target),
            group: vec![target],
            origin: Some(transform.translation),
            direction: Vec3::new(1.0, -0.2, 0.0),
            channels: presets::channels::WEAPON,
            ..default()
        },
    });
}
