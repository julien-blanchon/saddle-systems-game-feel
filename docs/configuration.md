# Configuration

## `GameFeelConfig`

| Field | Type | Default | Valid Range | Effect | Practical Advice |
| --- | --- | --- | --- | --- | --- |
| `overlay_flash_color` | `Color` | `WHITE` | any color | Reserved tint for screen-flash presentation | Leave at white unless your project wants a strong global tint bias |
| `overlay_vignette_color` | `Color` | `BLACK` | any color | Color written to the screen-edge vignette overlay | Use black or a dark brand color |
| `overlay_border_fraction` | `f32` | `0.18` | `0.0..=0.5` | Fraction of the screen covered by each vignette border quad | `0.12..0.22` reads well for most 16:9 scenes |
| `overlay_z_index` | `i32` | `1000` | any `i32` | UI z-index used by the screen overlay root | Raise this only if your project has very high-priority HUD layers |

## `GameFeelChannels`

`GameFeelChannels` is a bitflag wrapper used by listeners and recipe contexts.

Built-in constants:

- `GameFeelChannels::GAMEPLAY`
- `GameFeelChannels::AMBIENT`
- `GameFeelChannels::WEAPON`
- `GameFeelChannels::UI`
- `GameFeelChannels::ALL`
- `GameFeelChannels::NONE`

Guidance:

- use `WEAPON` for recoil and hit-confirm feedback
- use `GAMEPLAY` for collisions, landings, and enemy impacts
- use `AMBIENT` for rumbles or long-tail environmental feel
- use `UI` for reward pulses or icon emphasis

## `ShakeAccessibility`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `multiplier` | `f32` | `1.0` | usually `0.0..=1.0`, but values `> 1.0` are allowed | Global accessibility multiplier applied to all shake listeners |

Set `0.0` to disable shake completely while leaving the rest of the runtime active.

## `ShakeBudget`

| Field | Type | Default | Valid Range | Effect | Practical Advice |
| --- | --- | --- | --- | --- | --- |
| `max_trauma_per_frame` | `f32` | `0.65` | `0.0..=1.0` | Maximum newly accepted trauma in one frame | Lower this if many simultaneous hits become unreadable |
| `fatigue_gain_per_trauma` | `f32` | `0.85` | `>= 0` | How quickly accepted trauma increases listener fatigue | Raise it for rapid-fire weapons |
| `fatigue_decay_per_second` | `f32` | `1.8` | `>= 0` | Recovery rate of fatigue | Lower it for longer shake exhaustion |
| `minimum_acceptance` | `f32` | `0.15` | `0.0..=1.0` | Floor on trauma acceptance after fatigue | Keep above zero so large impacts still read |

## `ShakeProfile`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `translation_amplitude` | `Vec3` | `(0.25, 0.25, 0.10)` | per-axis authored gain | Maximum local translation offset before trauma scaling |
| `rotation_amplitude` | `Vec3` | `(0.04, 0.04, 0.07)` | radians per axis | Maximum local rotational offset before trauma scaling |
| `frequency_hz` | `f32` | `18.0` | `> 0` | Noise sampling rate |
| `decay_per_second` | `f32` | `1.8` | `>= 0` | Trauma decay rate |
| `trauma_power` | `f32` | `2.0` | `> 0` | Nonlinear trauma-to-amplitude curve |
| `directional_bias_decay_per_second` | `f32` | `8.0` | `>= 0` | How quickly directional recoil bias fades |
| `budget` | `ShakeBudget` | `ShakeBudget::default()` | nested | Saturation control for overlapping shake |
| `time_domain` | `EffectTimeDomain` | `Unscaled` | enum | Clock used to advance the shake state |

Zeroing one axis in either amplitude vector effectively disables it.

## `PunchProfile`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `translation_gain` | `Vec3` | `(1, 1, 1)` | Per-axis gain on translational impulses |
| `rotation_gain` | `Vec3` | `(1, 1, 1)` | Per-axis gain on rotational impulses |
| `fov_gain` | `f32` | `1.0` | Gain on perspective FOV punch |
| `spring` | `SpringSettings` | critically damped `7.5 Hz` | Spring return tuning |
| `time_domain` | `EffectTimeDomain` | `Unscaled` | Clock used to advance the spring |

## `SpringSettings`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `frequency_hz` | `f32` | `7.5` | `> 0` | Return speed |
| `damping_ratio` | `f32` | `1.0` | `>= 0` | Overshoot vs damping behavior |

Guidance:

- `1.0` is critically damped and safe for recoil
- values below `1.0` overshoot more
- values above `1.0` feel heavy and over-damped

## `EffectTimeDomain`

| Variant | Effect |
| --- | --- |
| `Unscaled` | Advances from real unscaled delta tracked by `GlobalTimeScale` |
| `GlobalScaled` | Advances from the resolved global scale after ramps and hitstop |

## `GlobalTimeScale`

`GlobalTimeScale` is a runtime resource, but it is also part of the crate's public configuration surface because consumer systems read it directly.

| Field | Meaning |
| --- | --- |
| `frame_index` | Monotonic frame counter owned by the runtime |
| `unscaled_delta_secs` | Current frame's unscaled delta |
| `elapsed_unscaled_secs` | Total unscaled elapsed time |
| `base_scale` | Resolved ramp scale before hitstop |
| `hitstop_scale` | Current hitstop contribution |
| `scale` | Final resolved global scale |
| `scaled_delta_secs` | Unscaled delta multiplied by `scale` |
| `elapsed_scaled_secs` | Total resolved scaled time |

## `EntityTimeScale`

| Field | Meaning |
| --- | --- |
| `base_scale` | Per-entity ramp scale before hitstop |
| `hitstop_scale` | Per-entity hitstop contribution |
| `scale` | Final resolved entity-local scale |

Pair `EntityTimeScale` with `resolve_effective_time_scale(...)` when consumer movement or animation systems need both global and local feel timing.

## Flash And Screen Pulse Requests

`RequestFlash` is the main authored flash surface:

| Field | Effect |
| --- | --- |
| `target` | Entity flash, screen flash, or both |
| `color` | Flash/pulse tint |
| `intensity` | Entity flash intensity or screen flash alpha |
| `chromatic_aberration` | Screen pulse aberration target |
| `vignette` | Screen pulse vignette target |
| `origin` | Optional world-space origin for distance attenuation |
| `attenuation` | Optional falloff from `origin` |
| `duration_secs` | Envelope duration |
| `easing` | Envelope curve |
| `clock` | `Unscaled` or `GlobalScaled` timing |

## Squash / Stretch Requests

`RequestSquashStretch` fields:

| Field | Effect |
| --- | --- |
| `peak_scale` | Peak relative or absolute scale |
| `mode` | `Relative` multiplies authored scale, `Absolute` targets an authored world scale |
| `duration_secs` | Full squash-plus-return duration |
| `easing` | Envelope curve |
| `clock` | Timing domain |
| `stacking` | `Multiply`, `Strongest`, or `Replace` |
| `direction` | Optional world/local direction used for directional stretch |
| `directional_magnitude` | Extra directional stretch amount |

## Recipes

The built-in `FeedbackRecipeLibrary::default()` includes:

- `light_hit`
- `heavy_impact`
- `explosion`
- `reward_ping`

Recipe authoring types:

- `FeedbackRecipe`
- `FeedbackStep`
- `FeedbackAction`
- `RecipeTrauma`
- `RecipeImpulse`
- `RecipeFlash`
- `RecipeHitstop`
- `RecipeTimeScale`
- `RecipeSquashStretch`

Important defaults:

- `FeedbackRecipeLibrary::default()` loads the built-in presets
- `FeedbackRecipeLibrary::empty()` gives a truly blank library

## Tuning Notes

### Start Conservative

Good defaults are intentionally moderate. If you need more impact:

1. raise shake trauma before raising raw amplitude
2. lengthen hitstop recovery before adding more hold frames
3. raise punch `frequency_hz` or lower damping before raising raw impulse vectors too far
4. use short screen flashes with clear color ownership instead of stacking many opaque pulses

### Separate Gameplay And Reward Feel

Use channels aggressively:

- gameplay cameras can subscribe to `GAMEPLAY | WEAPON`
- UI cameras or overlays can subscribe to `UI`
- spectator or cinematic cameras can apply their own listener scaling

### Slow Motion Integration

If gameplay systems should obey feel timing:

1. read `GlobalTimeScale` or `EntityTimeScale`
2. compute an effective scale with `resolve_effective_time_scale(...)`
3. multiply your own authored motion or animation clocks by that value

That keeps ownership explicit and avoids hidden global-time side effects.

