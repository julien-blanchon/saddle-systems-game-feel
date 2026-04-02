use super::*;
use crate::{GameFeelDiagnostics, GlobalTimeScale};

#[test]
fn mix_color_blends_between_base_and_flash() {
    let mixed = mix_color(Color::BLACK, Color::WHITE, 0.25).to_srgba();
    assert!((mixed.red - 0.25).abs() < 0.000_1);
    assert!((mixed.green - 0.25).abs() < 0.000_1);
    assert!((mixed.blue - 0.25).abs() < 0.000_1);
}

#[test]
fn entity_flash_output_uses_strongest_effect() {
    let mut app = App::new();
    app.init_resource::<GlobalTimeScale>();
    app.init_resource::<GameFeelDiagnostics>();
    app.add_systems(Update, update_entity_flash_outputs);

    let entity = app
        .world_mut()
        .spawn((
            FlashOutput::default(),
            EntityFlashRuntime {
                effects: vec![
                    ActiveEntityFlash {
                        color: Color::srgb(0.8, 0.2, 0.2),
                        intensity: 0.2,
                        tween: Tween::default(),
                        clock: EffectTimeDomain::Unscaled,
                        elapsed_secs: 0.0,
                    },
                    ActiveEntityFlash {
                        color: Color::srgb(1.0, 1.0, 1.0),
                        intensity: 0.6,
                        tween: Tween::default(),
                        clock: EffectTimeDomain::Unscaled,
                        elapsed_secs: 0.0,
                    },
                ],
            },
        ))
        .id();

    app.world_mut()
        .resource_mut::<GlobalTimeScale>()
        .unscaled_delta_secs = 0.05;
    app.update();

    let output = app
        .world()
        .get::<FlashOutput>(entity)
        .expect("flash output should exist");
    assert_eq!(output.color, Color::srgb(1.0, 1.0, 1.0));
    assert!(output.intensity > 0.08);
}
