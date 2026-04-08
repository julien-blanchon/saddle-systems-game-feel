# Game Feel Lab

Crate-local standalone lab app for validating the shared `saddle-systems-game-feel` crate in a real Bevy application.

## Purpose

- verify the shared feedback runtime in a deterministic, self-running scene
- expose global time scale, peak shake/flash metrics, recipe-step counts, and drift recovery through an on-screen overlay
- provide a BRP-ready and E2E-ready target without relying on project sandboxes

## Status

Working

## Run

```bash
cargo run -p saddle-systems-game-feel-lab
```

## E2E

```bash
cargo run -p saddle-systems-game-feel-lab --features e2e -- smoke_launch
cargo run -p saddle-systems-game-feel-lab --features e2e -- shake_focus
cargo run -p saddle-systems-game-feel-lab --features e2e -- hitstop_flash
cargo run -p saddle-systems-game-feel-lab --features e2e -- recipe_showcase
cargo run -p saddle-systems-game-feel-lab --features e2e -- snap_restored_state
cargo run -p saddle-systems-game-feel-lab --features e2e -- combo_recipe
cargo run -p saddle-systems-game-feel-lab --features e2e -- time_scale_pulse
cargo run -p saddle-systems-game-feel-lab --features e2e -- recoil_3d_punch
cargo run -p saddle-systems-game-feel-lab --features e2e -- comparison_toggle
cargo run -p saddle-systems-game-feel-lab --features e2e -- debug_cycle_presets
```

## BRP

Recommended in one terminal:

```bash
cargo run -p saddle-systems-game-feel-lab
```

Then in another terminal:

```bash
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_systems_game_feel::time_scale::GlobalTimeScale
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_systems_game_feel::config::GameFeelDiagnostics
uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/saddle_systems_game_feel_lab.png
uv run --project .codex/skills/bevy-brp/script brp extras shutdown
```

The helper launcher also works when its background mode is stable:

```bash
uv run --project .codex/skills/bevy-brp/script brp app launch saddle-systems-game-feel-lab
```

## Notes

- The lab keeps a single stable 2D scene so screenshots stay comparable across shake, hitstop, and recipe runs.
- The blue companion orb is marked to ignore global time scaling, which makes hitstop recovery visually obvious.
- Scenario setup resets the mode and evidence resources so each E2E run starts from a deterministic baseline.
