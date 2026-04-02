# Saddle Systems Game Feel

Reusable Bevy feedback runtime for impactful moment-to-moment response: trauma shake, hitstop, camera punch, time scaling, flash pulses, squash and stretch, and data-driven recipes.

The crate stays project-agnostic. It does not depend on `game_core`, `Screen`, `GameSet`, asset paths, or any gameplay vocabulary. Consumer crates emit generic feedback messages, and `saddle-systems-game-feel` handles stacking, timing, cleanup, and the built-in transform/sprite/screen adapters.

For always-on examples, tools, or sandboxes, `GameFeelPlugin::always_on(Update)` is the simplest entrypoint. For real games, prefer `GameFeelPlugin::new(...)` so activation and teardown stay aligned with your own schedules.

## Quick Start

```toml
[dependencies]
bevy = "0.18"
saddle-systems-game-feel = { git = "https://github.com/julien-blanchon/saddle-systems-game-feel" }
```

```rust,no_run
use bevy::prelude::*;
use saddle_systems_game_feel::*;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DemoState {
    #[default]
    Gameplay,
}

#[derive(Component)]
struct DemoCamera;

#[derive(Component)]
struct DemoTarget;

#[derive(Resource)]
struct TriggerTimer(Timer);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<DemoState>()
        .insert_resource(TriggerTimer(Timer::from_seconds(
            0.85,
            TimerMode::Repeating,
        )))
        .add_plugins(GameFeelPlugin::new(
            OnEnter(DemoState::Gameplay),
            OnExit(DemoState::Gameplay),
            Update,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, trigger_hit)
        .run();
}

fn setup(mut commands: Commands) {
    let camera = commands
        .spawn((
            Name::new("Gameplay Camera"),
            DemoCamera,
            Camera2d,
            ShakeListener::default(),
            PunchListener::default(),
            ScreenPulseListener::default(),
            Transform::from_xyz(0.0, 0.0, 10.0),
        ))
        .id();

    commands.spawn((
            Name::new("Target"),
            DemoTarget,
            Sprite::from_color(Color::srgb(0.9, 0.3, 0.2), Vec2::splat(96.0)),
            Transform::default(),
        ));

    let _ = camera;
}

fn trigger_hit(
    time: Res<Time>,
    mut timer: ResMut<TriggerTimer>,
    camera: Query<Entity, With<DemoCamera>>,
    target: Query<Entity, With<DemoTarget>>,
    mut trauma: MessageWriter<AddTrauma>,
    mut hitstop: MessageWriter<RequestHitstop>,
    mut flash: MessageWriter<RequestFlash>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let Ok(camera) = camera.single() else {
        return;
    };
    let Ok(target) = target.single() else {
        return;
    };
    trauma.write(AddTrauma {
        target: ListenerTarget::Entity(camera),
        trauma: 0.28,
        origin: None,
        attenuation: None,
        propagation_speed: None,
        directional_bias: Vec3::new(0.08, -0.02, 0.0),
        profile_override: None,
    });
    hitstop.write(RequestHitstop {
        target: TimeScaleTarget::World,
        hold_frames: 3,
        recovery_frames: 4,
        ..RequestHitstop::new(TimeScaleTarget::World, 3)
    });
    flash.write(RequestFlash {
        target: FlashTarget::EntityAndScreen {
            entity: target,
            screen: ListenerTarget::Entity(camera),
        },
        color: Color::WHITE,
        intensity: 0.45,
        chromatic_aberration: 0.05,
        vignette: 0.12,
        duration_secs: 0.16,
        easing: bevy::math::curve::easing::EaseFunction::SineOut,
        clock: EffectTimeDomain::Unscaled,
        origin: None,
        attenuation: None,
    });
}
```

## Public API

| Type | Purpose |
| --- | --- |
| `GameFeelPlugin` | Registers the runtime with injectable activate, deactivate, and update schedules |
| `GameFeelSystems` | Public ordering hooks: `ProcessRequests`, `UpdateSimulation`, `ApplyOutputs`, `Cleanup` |
| `AddTrauma` | Request trauma-based shake on listeners |
| `RequestCameraImpulse` | Request spring-damped translational / rotational / FOV punch |
| `RequestHitstop` / `RequestSplitHitstop` | Frame-counted freeze requests for world or targeted entities |
| `RequestTimeScale` | Ramp/hold/recover time-scale requests for world or targeted entities |
| `RequestFlash` | Entity flash plus screen pulse request with optional distance attenuation |
| `RequestSquashStretch` | Message-driven scale feedback on a target entity |
| `PlayFeedbackRecipe` | Single trigger message for named feedback recipes |
| `ShakeListener`, `PunchListener`, `ScreenPulseListener` | Opt-in listeners for shake, punch, and screen effects |
| `GlobalTimeScale`, `EntityTimeScale` | Public timing surfaces for consumer gameplay systems |
| `ShakeState`, `PunchState`, `FlashOutput`, `ScreenPulseOutput`, `SquashStretchState` | Inspectable output surfaces and adapter bridge points |
| `FeedbackRecipeLibrary`, `FeedbackRecipe`, `FeedbackAction` | Data-driven recipe authoring surface |
| `Tween`, `TweenRepeat`, `AttackSustainDecay` | Lightweight easing and envelope helpers reused internally and available to consumers |

## Stacking Rules

| Family | Rule |
| --- | --- |
| Trauma shake | Additive with clamp to `[0, 1]`, per-frame budget, and fatigue damping |
| Punch | Velocity impulses sum; the spring returns to baseline |
| Hitstop | Explicit policy per request: `Refresh`, `Max`, `AdditiveWithCap`, `IgnoreWeaker` |
| Time scale | Highest priority wins; ties resolve to the lowest scale |
| Entity flash | Strongest weighted intensity wins |
| Screen pulse | Flash color uses the strongest pulse; chromatic and vignette use max composition |
| Squash/stretch | Configurable `Multiply`, `Strongest`, or `Replace` stacking |

## Time Semantics

- `GlobalTimeScale` is a crate-owned timing surface, not a mutation of Bevy's global virtual time.
- `EntityTimeScale` is the per-entity equivalent for consumer systems that need local slow-mo or exemption handling.
- Every authored effect chooses `EffectTimeDomain::Unscaled` or `EffectTimeDomain::GlobalScaled`.
- Hitstop uses deterministic frame counts, while punch, flash, and scale effects advance on explicit time domains.

This keeps the crate generic: it can drive feel effects even when a project has its own fixed-step or pause model.

## Built-In Adapters

The runtime ships with a generic core plus a few concrete adapters:

- transform application for `ShakeState`, `PunchState`, and `SquashStretchState`
- perspective FOV punch for `Projection::Perspective`
- sprite-color flash application from `FlashOutput`
- screen overlay and chromatic aberration application from `ScreenPulseOutput`

`FlashOutput` remains the material-style extension point for projects that want custom shader or material adapters.

## Examples

| Example | Purpose | Run |
| --- | --- | --- |
| `basic` | Minimal self-running shake and punch loop | `cargo run -p saddle-systems-game-feel-example-basic` |
| `hitstop` | Hitstop plus flash plus squash on a moving target | `cargo run -p saddle-systems-game-feel-example-hitstop` |
| `recipes` | Built-in recipe playback loop | `cargo run -p saddle-systems-game-feel-example-recipes` |
| `time_scale` | World slow-mo with an entity that ignores global scaling | `cargo run -p saddle-systems-game-feel-example-time_scale` |
| `recoil_3d` | Perspective-camera recoil example proving 3D transform and FOV punch support | `cargo run -p saddle-systems-game-feel-example-recoil_3d` |
| `debug_showcase` | Rich self-running showcase of recoil, impact, explosion, and reward pulses | `cargo run -p saddle-systems-game-feel-example-debug_showcase` |

## Workspace Lab

The crate-local lab lives at `shared/systems/saddle-systems-game-feel/examples/lab`:

```bash
cargo run -p saddle-systems-game-feel-lab
```

It is the primary BRP and E2E verification target for this crate.

## More Docs

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)

## Current Limitations

- The built-in flash adapter only writes `Sprite` color directly. Material or shader-backed flash should read `FlashOutput` and apply it in project-specific render code.
- Time scaling is exposed as public resources/components; the crate does not mutate Bevy's global `Time<Virtual>` or freeze arbitrary gameplay systems automatically.
- The crate now ships both 2D showcase examples and a focused 3D recoil example, but the crate-local lab still concentrates on one deterministic 2D scene so E2E screenshots stay comparable.
- Projects that need custom 3D material flashes or heavier renderer-specific post-processing will still usually add project-specific adapters on top of `FlashOutput` or `ScreenPulseOutput`.
