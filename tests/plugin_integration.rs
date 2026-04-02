use bevy::{
    app::PostStartup,
    ecs::{message::Messages, schedule::ScheduleLabel},
    prelude::*,
    time::TimeUpdateStrategy,
};

use saddle_systems_game_feel::{
    AddTrauma, GameFeelChannels, GameFeelPlugin, GameFeelSystems, IgnoreHitstop, ListenerTarget,
    PunchListener, RequestCameraImpulse, RequestHitstop, ShakeListener, ShakeState,
};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct ActivateSchedule;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct DeactivateSchedule;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct SimulationSchedule;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AfterFeel;

#[derive(Resource, Default)]
struct OrderLog(Vec<&'static str>);

fn start_runtime(app: &mut App) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f64(1.0 / 60.0),
    ));
    app.finish();
    app.world_mut().run_schedule(PostStartup);
}

fn advance_frame(app: &mut App) {
    app.world_mut()
        .resource_mut::<Time<Real>>()
        .advance_by(std::time::Duration::from_secs_f32(1.0 / 60.0));
    app.update();
}

#[test]
fn plugin_exposes_ordering_points_on_custom_schedule() {
    fn mark_feel(mut log: ResMut<OrderLog>) {
        log.0.push("feel");
    }

    fn mark_after(mut log: ResMut<OrderLog>) {
        log.0.push("after");
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_schedule(ActivateSchedule)
        .init_schedule(DeactivateSchedule)
        .init_schedule(SimulationSchedule)
        .init_resource::<OrderLog>()
        .add_plugins(GameFeelPlugin::new(
            ActivateSchedule,
            DeactivateSchedule,
            SimulationSchedule,
        ))
        .configure_sets(
            SimulationSchedule,
            GameFeelSystems::UpdateSimulation.before(AfterFeel),
        )
        .add_systems(
            SimulationSchedule,
            (
                mark_feel.in_set(GameFeelSystems::UpdateSimulation),
                mark_after.in_set(AfterFeel),
            ),
        );

    app.finish();
    app.world_mut().run_schedule(ActivateSchedule);
    app.world_mut().run_schedule(SimulationSchedule);

    assert_eq!(app.world().resource::<OrderLog>().0, vec!["feel", "after"]);
}

#[test]
fn trauma_requests_only_affect_matching_channels() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_schedule(ActivateSchedule)
        .init_schedule(DeactivateSchedule)
        .init_schedule(SimulationSchedule)
        .add_plugins(GameFeelPlugin::new(
            ActivateSchedule,
            DeactivateSchedule,
            SimulationSchedule,
        ));

    let matching = app
        .world_mut()
        .spawn((
            ShakeListener {
                channels: GameFeelChannels::WEAPON,
                ..default()
            },
            Transform::default(),
        ))
        .id();
    let non_matching = app
        .world_mut()
        .spawn((
            ShakeListener {
                channels: GameFeelChannels::GAMEPLAY,
                ..default()
            },
            Transform::default(),
        ))
        .id();

    app.finish();
    app.world_mut().run_schedule(ActivateSchedule);
    app.world_mut()
        .resource_mut::<Time<Real>>()
        .advance_by(std::time::Duration::from_secs_f32(1.0 / 60.0));
    app.world_mut()
        .resource_mut::<Messages<AddTrauma>>()
        .write(AddTrauma {
            target: ListenerTarget::Channels(GameFeelChannels::WEAPON),
            trauma: 0.35,
            origin: None,
            attenuation: None,
            propagation_speed: None,
            directional_bias: Vec3::new(0.1, 0.0, 0.0),
            profile_override: None,
        });
    app.world_mut().run_schedule(SimulationSchedule);

    assert!(app.world().entity(matching).contains::<ShakeState>());
    assert!(!app.world().entity(non_matching).contains::<ShakeState>());
}

#[test]
fn ignore_hitstop_marker_rejects_targeted_freeze() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(GameFeelPlugin::always_on(Update));

    let entity = app.world_mut().spawn((IgnoreHitstop,)).id();
    start_runtime(&mut app);
    app.world_mut()
        .resource_mut::<Messages<RequestHitstop>>()
        .write(RequestHitstop::new(
            saddle_systems_game_feel::TimeScaleTarget::Entity(entity),
            3,
        ));
    app.update();

    assert!(
        !app.world()
            .entity(entity)
            .contains::<saddle_systems_game_feel::EntityTimeScale>()
    );
}

#[test]
fn world_hitstop_drops_global_scale_and_recovers() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(GameFeelPlugin::always_on(Update));

    start_runtime(&mut app);
    app.world_mut()
        .resource_mut::<Messages<RequestHitstop>>()
        .write(RequestHitstop {
            target: saddle_systems_game_feel::TimeScaleTarget::World,
            hold_frames: 2,
            recovery_frames: 2,
            ..RequestHitstop::new(saddle_systems_game_feel::TimeScaleTarget::World, 2)
        });

    advance_frame(&mut app);
    assert!(
        app.world()
            .resource::<saddle_systems_game_feel::GlobalTimeScale>()
            .scale
            < 0.001
    );

    advance_frame(&mut app);
    advance_frame(&mut app);
    assert!(
        app.world()
            .resource::<saddle_systems_game_feel::GlobalTimeScale>()
            .scale
            < 0.001
    );

    advance_frame(&mut app);
    assert!(
        app.world()
            .resource::<saddle_systems_game_feel::GlobalTimeScale>()
            .scale
            > 0.45
    );

    advance_frame(&mut app);
    assert!(
        app.world()
            .resource::<saddle_systems_game_feel::GlobalTimeScale>()
            .scale
            > 0.99
    );
}

#[test]
fn transform_returns_to_baseline_after_punch_finishes() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(GameFeelPlugin::always_on(Update));

    let entity = app
        .world_mut()
        .spawn((PunchListener::default(), Transform::from_xyz(1.0, 2.0, 3.0)))
        .id();

    start_runtime(&mut app);
    app.world_mut()
        .resource_mut::<Messages<RequestCameraImpulse>>()
        .write(RequestCameraImpulse::new(
            ListenerTarget::Entity(entity),
            Vec3::new(0.4, 0.0, 0.0),
        ));

    for _ in 0..120 {
        app.update();
    }

    let transform = app
        .world()
        .get::<Transform>(entity)
        .expect("transform should exist");
    assert!((transform.translation - Vec3::new(1.0, 2.0, 3.0)).length() < 0.001);
    assert!((transform.scale - Vec3::ONE).length() < 0.001);
}
