# Architecture

## Layering

`saddle-systems-game-feel` is split into pure effect math plus a thin Bevy runtime layer.

Pure or mostly-pure modules:

1. `tween.rs`
2. `config.rs`
3. `channels.rs`
4. the math-heavy helpers inside `shake.rs`, `punch.rs`, `squash.rs`, and `time_scale.rs`

Bevy-facing runtime modules:

1. `messages.rs`
2. `recipe.rs`
3. `lib.rs`
4. the queue/runtime/output systems in `shake.rs`, `punch.rs`, `flash.rs`, `squash.rs`, and `time_scale.rs`

The pure layer owns stacking, attenuation, spring behavior, tween sampling, and time-scale resolution. The Bevy layer only reads messages, advances runtime state, applies outputs, and cleans up adapters.

## Runtime Flow

```text
Gameplay messages
  -> recipe playback (optional)
  -> effect request processing
  -> shake / punch / time-scale / flash / rumble / squash runtimes
  -> public output components/resources
  -> built-in adapters (transform, projection, sprite, screen pulse)
  -> diagnostics
```

More concretely:

1. `AddTrauma`, `RequestCameraImpulse`, `RequestHitstop`, `RequestTimeScale`, `RequestFlash`, `RequestRumble`, `RequestSquashStretch`, and `PlayFeedbackRecipe` enter through the message layer.
2. Recipes expand into concrete request messages.
3. Request processors resolve channels, attenuation, and per-target runtime state.
4. Simulation systems advance effect queues and sampled outputs.
5. Built-in adapters apply those outputs additively and reversibly.
6. Cleanup systems drop empty runtime containers and maintain overlay ownership.

Screen pulses now intentionally support two presentation modes:

- `ScreenPulsePresentation::OutputOnly` keeps `ScreenPulseOutput` as pure data for downstream renderer crates.
- `ScreenPulsePresentation::LegacyBuiltIn` also enables the crate's older UI-overlay and `ChromaticAberration` writeback path.

## Schedule Ordering

`GameFeelSystems` is intentionally public:

1. `ProcessRequests`
2. `UpdateSimulation`
3. `ApplyOutputs`
4. `Cleanup`

When the plugin runs on the normal `Update` schedule, the built-in presentation restore step happens in `PreUpdate` and the adapter writeback happens in `PostUpdate`. That keeps normal gameplay logic from seeing permanently polluted transforms.

When the caller uses a non-`Update` schedule, the crate falls back to running restore and apply work inside the injected schedule itself. This preserves injectable schedules while still keeping the public ordering hooks usable in sandboxes or custom simulation loops.

## Adapter Model

The runtime is deliberately split between generic outputs and concrete adapters.

Generic output surfaces:

- `GlobalTimeScale`
- `EntityTimeScale`
- `ShakeState`
- `PunchState`
- `FlashOutput`
- `ScreenPulseOutput`
- `RumbleOutput`
- `SquashStretchState`

Built-in adapters:

- `Transform` writeback for shake, punch, and squash/stretch
- `Projection::Perspective` FOV writeback for punch
- `Sprite` flash writeback for `FlashOutput`
- optional UI-overlay and `ChromaticAberration` writeback for `ScreenPulseOutput`
- no built-in platform haptics backend; `RumbleOutput` intentionally stays as a portable output surface for downstream gamepad or platform adapters

This means downstream projects can:

- use the built-in adapters directly
- ignore them and read the output surfaces themselves
- mix both patterns in one game
- keep game-feel authored separately while routing screen pulses into another crate's unified screen-effects stack

## Stacking Behavior

### Shake

- Trauma is additive and clamped to `[0, 1]`.
- Per-frame budget limits how much new trauma can land in one frame.
- Fatigue reduces acceptance when many sources stack aggressively.
- Directional bias is additive and decays separately from scalar trauma.

### Punch

- Impulses add into velocity state.
- The return motion is spring-damped, not a fixed interpolation.
- Local-space and world-space impulses coexist.

### Time Scale

- Ramps are resolved by priority.
- Equal-priority ramps choose the lowest scale.
- Hitstop is orthogonal to ramps and composes multiplicatively.
- `IgnoreGlobalTimeScale` and `IgnoreHitstop` let consumer systems opt out selectively.

### Flash

- Entity flash picks the strongest weighted flash each frame.
- Screen flash picks the strongest flash alpha for color ownership.
- Chromatic aberration and vignette use max composition.

### Squash / Stretch

- `Multiply` composes all active scale multipliers.
- `Strongest` picks the effect furthest from `Vec3::ONE`.
- `Replace` uses the most recently added effect only.

## Time Domains

Every authored effect chooses one of two clocks:

- `Unscaled`
- `GlobalScaled`

Use `Unscaled` for:

- flashes that should keep fading during hitstop
- recoil tails that should finish even while the world is slowed
- UI or reward pulses

Use `GlobalScaled` for:

- effects that should slow down together with gameplay
- follow-through that should feel stretched by slow motion

Hitstop itself is frame-counted and deterministic. It does not depend on wall-clock frame rate.

## Cleanup And Drift Protection

Presentation adapters always restore the previous frame's authored transform/projection/sprite state before writing the new output. This prevents:

- permanent transform drift
- flash color accumulation
- projection FOV drift after punch

The cleanup stage only removes empty runtime containers. Public output components are allowed to remain present with zeroed values so consumer adapters can choose stable query shapes if they prefer them.

## Testing Strategy

The crate verifies the boundary in four layers:

- pure unit tests for tweening, attenuation, hitstop stacking, springs, and scale helpers
- App-level integration tests for schedule injection, message entrypoints, and drift recovery
- runnable standalone examples for focused manual smoke checks
- a crate-local lab with E2E scenarios and screenshot gates
