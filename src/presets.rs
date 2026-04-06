use bevy::{math::curve::easing::EaseFunction, prelude::*};

pub mod channels {
    use crate::GameFeelChannels;

    pub const GAMEPLAY: GameFeelChannels = GameFeelChannels::new(1 << 0);
    pub const AMBIENT: GameFeelChannels = GameFeelChannels::new(1 << 1);
    pub const WEAPON: GameFeelChannels = GameFeelChannels::new(1 << 2);
    pub const UI: GameFeelChannels = GameFeelChannels::new(1 << 3);
}

pub mod recipes {
    use super::*;
    use crate::{
        DistanceAttenuation, EffectTimeDomain, EntitySelector, FeedbackAction, FeedbackCondition,
        FeedbackRecipe, FeedbackRecipeLibrary, FeedbackRecipeRepeat, FeedbackStep,
        ListenerSelector, RecipeFlash, RecipeFlashTarget, RecipeHitstop, RecipeHooks,
        RecipeImpulse, RecipeRumble, RecipeSquashStretch, RecipeTimeScale, RecipeTrauma,
        TimeScaleSelector,
        punch::ImpulseSpace,
        squash::{ScaleEffectMode, ScaleStacking},
        time_scale::HitstopStacking,
    };

    pub const LIGHT_HIT: &str = "light_hit";
    pub const HEAVY_IMPACT: &str = "heavy_impact";
    pub const EXPLOSION: &str = "explosion";
    pub const REWARD_PING: &str = "reward_ping";
    pub const WEAPON_FIRE: &str = "weapon_fire";
    pub const LANDING_IMPACT: &str = "landing_impact";
    pub const DASH_BURST: &str = "dash_burst";
    pub const PARRY: &str = "parry";

    pub const NAMES: [&str; 8] = [
        LIGHT_HIT,
        HEAVY_IMPACT,
        EXPLOSION,
        REWARD_PING,
        WEAPON_FIRE,
        LANDING_IMPACT,
        DASH_BURST,
        PARRY,
    ];

    #[must_use]
    pub fn library() -> FeedbackRecipeLibrary {
        let mut library = FeedbackRecipeLibrary::default();
        insert_all(&mut library);
        library
    }

    pub fn insert_all(library: &mut FeedbackRecipeLibrary) {
        library.insert(LIGHT_HIT, light_hit());
        library.insert(HEAVY_IMPACT, heavy_impact());
        library.insert(EXPLOSION, explosion());
        library.insert(REWARD_PING, reward_ping());
        library.insert(WEAPON_FIRE, weapon_fire());
        library.insert(LANDING_IMPACT, landing_impact());
        library.insert(DASH_BURST, dash_burst());
        library.insert(PARRY, parry());
    }

    #[must_use]
    pub fn light_hit() -> FeedbackRecipe {
        FeedbackRecipe {
            cooldown_secs: 0.02,
            condition: FeedbackCondition::RequiresListener,
            repeat: FeedbackRecipeRepeat::Once,
            steps: vec![FeedbackStep {
                name: "impact".into(),
                at_secs: 0.0,
                actions: vec![
                    FeedbackAction::Trauma(RecipeTrauma {
                        target: ListenerSelector::ContextListener,
                        trauma: 0.16,
                        directional_bias: Vec3::new(0.08, -0.02, 0.0),
                        use_context_origin: true,
                        attenuation: None,
                        propagation_speed: None,
                    }),
                    FeedbackAction::CameraImpulse(RecipeImpulse {
                        target: ListenerSelector::ContextListener,
                        translation: Vec3::new(-0.07, 0.03, 0.0),
                        rotation: Vec3::new(0.0, 0.0, -0.06),
                        fov: 0.0,
                        use_context_origin: false,
                        attenuation: None,
                        propagation_speed: None,
                        space: ImpulseSpace::Local,
                    }),
                    FeedbackAction::Flash(RecipeFlash {
                        target: RecipeFlashTarget::EntityAndScreen {
                            entity: EntitySelector::ContextTarget,
                            screen: ListenerSelector::ContextListener,
                        },
                        color: Color::WHITE,
                        intensity: 0.35,
                        chromatic_aberration: 0.03,
                        vignette: 0.08,
                        use_context_origin: false,
                        attenuation: None,
                        duration_secs: 0.12,
                        easing: EaseFunction::SineOut,
                        clock: EffectTimeDomain::Unscaled,
                    }),
                    FeedbackAction::Rumble(RecipeRumble {
                        target: ListenerSelector::ContextListener,
                        low_frequency: 0.30,
                        high_frequency: 0.45,
                        duration_secs: 0.12,
                        easing: EaseFunction::SineOut,
                        clock: EffectTimeDomain::Unscaled,
                    }),
                    FeedbackAction::Hitstop(RecipeHitstop {
                        target: TimeScaleSelector::ContextTarget,
                        hold_frames: 2,
                        recovery_frames: 2,
                        stacking: HitstopStacking::Refresh,
                    }),
                    FeedbackAction::Hooks(RecipeHooks {
                        audio_cue: Some("impact_light".into()),
                        particle_cue: Some("impact_spark".into()),
                        target: Some(EntitySelector::ContextTarget),
                        use_context_origin: true,
                    }),
                ],
            }],
        }
    }

    #[must_use]
    pub fn heavy_impact() -> FeedbackRecipe {
        FeedbackRecipe {
            cooldown_secs: 0.08,
            condition: FeedbackCondition::RequiresListener,
            repeat: FeedbackRecipeRepeat::Once,
            steps: vec![
                FeedbackStep {
                    name: "freeze".into(),
                    at_secs: 0.0,
                    actions: vec![
                        FeedbackAction::Hitstop(RecipeHitstop {
                            target: TimeScaleSelector::ContextTarget,
                            hold_frames: 5,
                            recovery_frames: 3,
                            stacking: HitstopStacking::Refresh,
                        }),
                        FeedbackAction::Trauma(RecipeTrauma {
                            target: ListenerSelector::ContextListener,
                            trauma: 0.34,
                            directional_bias: Vec3::new(0.22, -0.10, 0.0),
                            use_context_origin: true,
                            attenuation: Some(DistanceAttenuation {
                                inner_radius: 0.0,
                                outer_radius: 20.0,
                                exponent: 1.2,
                            }),
                            propagation_speed: None,
                        }),
                        FeedbackAction::CameraImpulse(RecipeImpulse {
                            target: ListenerSelector::ContextListener,
                            translation: Vec3::new(-0.20, 0.08, 0.0),
                            rotation: Vec3::new(0.0, 0.0, -0.12),
                            fov: 0.03,
                            use_context_origin: false,
                            attenuation: None,
                            propagation_speed: None,
                            space: ImpulseSpace::Local,
                        }),
                        FeedbackAction::Flash(RecipeFlash {
                            target: RecipeFlashTarget::EntityAndScreen {
                                entity: EntitySelector::ContextTarget,
                                screen: ListenerSelector::ContextListener,
                            },
                            color: Color::srgb(1.0, 0.95, 0.9),
                            intensity: 0.58,
                            chromatic_aberration: 0.08,
                            vignette: 0.20,
                            use_context_origin: false,
                            attenuation: None,
                            duration_secs: 0.18,
                            easing: EaseFunction::SineOut,
                            clock: EffectTimeDomain::Unscaled,
                        }),
                        FeedbackAction::Rumble(RecipeRumble {
                            target: ListenerSelector::ContextListener,
                            low_frequency: 0.85,
                            high_frequency: 0.55,
                            duration_secs: 0.20,
                            easing: EaseFunction::SineOut,
                            clock: EffectTimeDomain::Unscaled,
                        }),
                        FeedbackAction::SquashStretch(RecipeSquashStretch {
                            target: EntitySelector::ContextTarget,
                            peak_scale: Vec3::new(1.18, 0.82, 1.0),
                            mode: ScaleEffectMode::Relative,
                            stacking: ScaleStacking::Multiply,
                            duration_secs: 0.18,
                            easing: EaseFunction::BackOut,
                            clock: EffectTimeDomain::Unscaled,
                            direction_from_context: true,
                        }),
                        FeedbackAction::Hooks(RecipeHooks {
                            audio_cue: Some("impact_heavy".into()),
                            particle_cue: Some("impact_burst".into()),
                            target: Some(EntitySelector::ContextTarget),
                            use_context_origin: true,
                        }),
                    ],
                },
                FeedbackStep {
                    name: "slow_mo_tail".into(),
                    at_secs: 0.04,
                    actions: vec![FeedbackAction::TimeScale(RecipeTimeScale {
                        target: TimeScaleSelector::World,
                        scale: 0.55,
                        ramp_in_secs: 0.02,
                        hold_secs: 0.10,
                        ramp_out_secs: 0.16,
                        easing: EaseFunction::SineInOut,
                        priority: 1,
                    })],
                },
            ],
        }
    }

    #[must_use]
    pub fn explosion() -> FeedbackRecipe {
        FeedbackRecipe {
            cooldown_secs: 0.15,
            condition: FeedbackCondition::RequiresListener,
            repeat: FeedbackRecipeRepeat::Once,
            steps: vec![
                FeedbackStep {
                    name: "blast".into(),
                    at_secs: 0.0,
                    actions: vec![
                        FeedbackAction::Trauma(RecipeTrauma {
                            target: ListenerSelector::ContextListener,
                            trauma: 0.46,
                            directional_bias: Vec3::new(0.0, 0.0, -0.04),
                            use_context_origin: true,
                            attenuation: Some(DistanceAttenuation {
                                inner_radius: 0.0,
                                outer_radius: 30.0,
                                exponent: 1.4,
                            }),
                            propagation_speed: Some(16.0),
                        }),
                        FeedbackAction::CameraImpulse(RecipeImpulse {
                            target: ListenerSelector::ContextListener,
                            translation: Vec3::new(0.0, 0.14, 0.30),
                            rotation: Vec3::new(-0.05, 0.02, 0.03),
                            fov: 0.05,
                            use_context_origin: true,
                            attenuation: Some(DistanceAttenuation {
                                inner_radius: 0.0,
                                outer_radius: 30.0,
                                exponent: 1.2,
                            }),
                            propagation_speed: Some(16.0),
                            space: ImpulseSpace::World,
                        }),
                        FeedbackAction::Flash(RecipeFlash {
                            target: RecipeFlashTarget::Screen(ListenerSelector::ContextListener),
                            color: Color::srgb(1.0, 0.72, 0.40),
                            intensity: 0.45,
                            chromatic_aberration: 0.10,
                            vignette: 0.24,
                            use_context_origin: true,
                            attenuation: Some(DistanceAttenuation {
                                inner_radius: 0.0,
                                outer_radius: 24.0,
                                exponent: 1.0,
                            }),
                            duration_secs: 0.26,
                            easing: EaseFunction::SineOut,
                            clock: EffectTimeDomain::Unscaled,
                        }),
                        FeedbackAction::Rumble(RecipeRumble {
                            target: ListenerSelector::ContextListener,
                            low_frequency: 1.0,
                            high_frequency: 0.65,
                            duration_secs: 0.40,
                            easing: EaseFunction::SineOut,
                            clock: EffectTimeDomain::Unscaled,
                        }),
                        FeedbackAction::Hooks(RecipeHooks {
                            audio_cue: Some("explosion_heavy".into()),
                            particle_cue: Some("explosion_debris".into()),
                            target: None,
                            use_context_origin: true,
                        }),
                    ],
                },
                FeedbackStep {
                    name: "rumble_tail".into(),
                    at_secs: 0.18,
                    actions: vec![FeedbackAction::TimeScale(RecipeTimeScale {
                        target: TimeScaleSelector::World,
                        scale: 0.72,
                        ramp_in_secs: 0.0,
                        hold_secs: 0.16,
                        ramp_out_secs: 0.28,
                        easing: EaseFunction::SineInOut,
                        priority: 0,
                    })],
                },
            ],
        }
    }

    #[must_use]
    pub fn reward_ping() -> FeedbackRecipe {
        FeedbackRecipe {
            cooldown_secs: 0.05,
            condition: FeedbackCondition::Always,
            repeat: FeedbackRecipeRepeat::Once,
            steps: vec![FeedbackStep {
                name: "pulse".into(),
                at_secs: 0.0,
                actions: vec![
                    FeedbackAction::Flash(RecipeFlash {
                        target: RecipeFlashTarget::Screen(ListenerSelector::ContextListener),
                        color: Color::srgb(1.0, 0.95, 0.4),
                        intensity: 0.25,
                        chromatic_aberration: 0.02,
                        vignette: 0.05,
                        use_context_origin: false,
                        attenuation: None,
                        duration_secs: 0.18,
                        easing: EaseFunction::SineOut,
                        clock: EffectTimeDomain::Unscaled,
                    }),
                    FeedbackAction::TimeScale(RecipeTimeScale {
                        target: TimeScaleSelector::World,
                        scale: 0.82,
                        ramp_in_secs: 0.0,
                        hold_secs: 0.04,
                        ramp_out_secs: 0.12,
                        easing: EaseFunction::SineInOut,
                        priority: 0,
                    }),
                    FeedbackAction::Hooks(RecipeHooks {
                        audio_cue: Some("reward_ping".into()),
                        particle_cue: Some("reward_sparkle".into()),
                        target: Some(EntitySelector::ContextTarget),
                        use_context_origin: true,
                    }),
                ],
            }],
        }
    }

    #[must_use]
    pub fn weapon_fire() -> FeedbackRecipe {
        FeedbackRecipe {
            cooldown_secs: 0.02,
            condition: FeedbackCondition::RequiresListener,
            repeat: FeedbackRecipeRepeat::Once,
            steps: vec![FeedbackStep {
                name: "fire".into(),
                at_secs: 0.0,
                actions: vec![
                    FeedbackAction::Trauma(RecipeTrauma {
                        target: ListenerSelector::ContextListener,
                        trauma: 0.10,
                        directional_bias: Vec3::new(0.0, 0.04, -0.02),
                        use_context_origin: false,
                        attenuation: None,
                        propagation_speed: None,
                    }),
                    FeedbackAction::CameraImpulse(RecipeImpulse {
                        target: ListenerSelector::ContextListener,
                        translation: Vec3::new(0.0, 0.02, -0.08),
                        rotation: Vec3::new(-0.03, 0.0, 0.0),
                        fov: -0.015,
                        use_context_origin: false,
                        attenuation: None,
                        propagation_speed: None,
                        space: ImpulseSpace::Local,
                    }),
                    FeedbackAction::Flash(RecipeFlash {
                        target: RecipeFlashTarget::Screen(ListenerSelector::ContextListener),
                        color: Color::srgb(1.0, 0.85, 0.5),
                        intensity: 0.15,
                        chromatic_aberration: 0.02,
                        vignette: 0.04,
                        use_context_origin: false,
                        attenuation: None,
                        duration_secs: 0.06,
                        easing: EaseFunction::SineOut,
                        clock: EffectTimeDomain::Unscaled,
                    }),
                    FeedbackAction::Rumble(RecipeRumble {
                        target: ListenerSelector::ContextListener,
                        low_frequency: 0.40,
                        high_frequency: 0.60,
                        duration_secs: 0.08,
                        easing: EaseFunction::SineOut,
                        clock: EffectTimeDomain::Unscaled,
                    }),
                    FeedbackAction::Hooks(RecipeHooks {
                        audio_cue: Some("weapon_fire".into()),
                        particle_cue: Some("muzzle_flash".into()),
                        target: None,
                        use_context_origin: true,
                    }),
                ],
            }],
        }
    }

    #[must_use]
    pub fn landing_impact() -> FeedbackRecipe {
        FeedbackRecipe {
            cooldown_secs: 0.05,
            condition: FeedbackCondition::RequiresTarget,
            repeat: FeedbackRecipeRepeat::Once,
            steps: vec![FeedbackStep {
                name: "land".into(),
                at_secs: 0.0,
                actions: vec![
                    FeedbackAction::Trauma(RecipeTrauma {
                        target: ListenerSelector::ContextListener,
                        trauma: 0.12,
                        directional_bias: Vec3::new(0.0, -0.06, 0.0),
                        use_context_origin: true,
                        attenuation: None,
                        propagation_speed: None,
                    }),
                    FeedbackAction::SquashStretch(RecipeSquashStretch {
                        target: EntitySelector::ContextTarget,
                        peak_scale: Vec3::new(1.25, 0.75, 1.0),
                        mode: ScaleEffectMode::Relative,
                        stacking: ScaleStacking::Multiply,
                        duration_secs: 0.22,
                        easing: EaseFunction::BackOut,
                        clock: EffectTimeDomain::Unscaled,
                        direction_from_context: false,
                    }),
                    FeedbackAction::Hooks(RecipeHooks {
                        audio_cue: Some("landing_thud".into()),
                        particle_cue: Some("dust_puff".into()),
                        target: Some(EntitySelector::ContextTarget),
                        use_context_origin: true,
                    }),
                ],
            }],
        }
    }

    #[must_use]
    pub fn dash_burst() -> FeedbackRecipe {
        FeedbackRecipe {
            cooldown_secs: 0.08,
            condition: FeedbackCondition::RequiresTarget,
            repeat: FeedbackRecipeRepeat::Once,
            steps: vec![FeedbackStep {
                name: "dash".into(),
                at_secs: 0.0,
                actions: vec![
                    FeedbackAction::Trauma(RecipeTrauma {
                        target: ListenerSelector::ContextListener,
                        trauma: 0.08,
                        directional_bias: Vec3::ZERO,
                        use_context_origin: false,
                        attenuation: None,
                        propagation_speed: None,
                    }),
                    FeedbackAction::SquashStretch(RecipeSquashStretch {
                        target: EntitySelector::ContextTarget,
                        peak_scale: Vec3::new(0.80, 1.30, 1.0),
                        mode: ScaleEffectMode::Relative,
                        stacking: ScaleStacking::Multiply,
                        duration_secs: 0.16,
                        easing: EaseFunction::BackOut,
                        clock: EffectTimeDomain::Unscaled,
                        direction_from_context: true,
                    }),
                    FeedbackAction::TimeScale(RecipeTimeScale {
                        target: TimeScaleSelector::World,
                        scale: 0.85,
                        ramp_in_secs: 0.0,
                        hold_secs: 0.03,
                        ramp_out_secs: 0.10,
                        easing: EaseFunction::SineInOut,
                        priority: 0,
                    }),
                    FeedbackAction::Hooks(RecipeHooks {
                        audio_cue: Some("dash_whoosh".into()),
                        particle_cue: Some("speed_lines".into()),
                        target: Some(EntitySelector::ContextTarget),
                        use_context_origin: true,
                    }),
                ],
            }],
        }
    }

    #[must_use]
    pub fn parry() -> FeedbackRecipe {
        FeedbackRecipe {
            cooldown_secs: 0.10,
            condition: FeedbackCondition::RequiresListener,
            repeat: FeedbackRecipeRepeat::Once,
            steps: vec![
                FeedbackStep {
                    name: "deflect".into(),
                    at_secs: 0.0,
                    actions: vec![
                        FeedbackAction::Hitstop(RecipeHitstop {
                            target: TimeScaleSelector::World,
                            hold_frames: 6,
                            recovery_frames: 3,
                            stacking: HitstopStacking::Refresh,
                        }),
                        FeedbackAction::Trauma(RecipeTrauma {
                            target: ListenerSelector::ContextListener,
                            trauma: 0.28,
                            directional_bias: Vec3::new(0.15, 0.0, 0.0),
                            use_context_origin: false,
                            attenuation: None,
                            propagation_speed: None,
                        }),
                        FeedbackAction::Flash(RecipeFlash {
                            target: RecipeFlashTarget::EntityAndScreen {
                                entity: EntitySelector::ContextTarget,
                                screen: ListenerSelector::ContextListener,
                            },
                            color: Color::srgb(0.8, 0.9, 1.0),
                            intensity: 0.70,
                            chromatic_aberration: 0.12,
                            vignette: 0.15,
                            use_context_origin: false,
                            attenuation: None,
                            duration_secs: 0.14,
                            easing: EaseFunction::SineOut,
                            clock: EffectTimeDomain::Unscaled,
                        }),
                        FeedbackAction::Rumble(RecipeRumble {
                            target: ListenerSelector::ContextListener,
                            low_frequency: 0.90,
                            high_frequency: 0.70,
                            duration_secs: 0.14,
                            easing: EaseFunction::SineOut,
                            clock: EffectTimeDomain::Unscaled,
                        }),
                        FeedbackAction::Hooks(RecipeHooks {
                            audio_cue: Some("parry_clang".into()),
                            particle_cue: Some("spark_burst".into()),
                            target: Some(EntitySelector::ContextTarget),
                            use_context_origin: true,
                        }),
                    ],
                },
                FeedbackStep {
                    name: "slow_mo".into(),
                    at_secs: 0.02,
                    actions: vec![FeedbackAction::TimeScale(RecipeTimeScale {
                        target: TimeScaleSelector::World,
                        scale: 0.20,
                        ramp_in_secs: 0.0,
                        hold_secs: 0.12,
                        ramp_out_secs: 0.20,
                        easing: EaseFunction::SineInOut,
                        priority: 2,
                    })],
                },
            ],
        }
    }
}
