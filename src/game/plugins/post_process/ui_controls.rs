use bevy::prelude::*;

/// Component for the settings UI panel
#[derive(Component)]
pub struct SettingsPanel;

/// Component for effect sliders
#[derive(Component)]
struct EffectSlider {
    effect_type: EffectType,
}

/// Types of effects that can be adjusted
#[derive(Clone, Copy)]
enum EffectType {
    Exposure,
    Bloom,
    ChromaticAberration,
    Vignette,
    FilmGrain,
    LensDistortion,
    ColorTemperature,
}

/// System to spawn the UI controls
pub fn spawn_ui_controls(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Px(10.0),
                    top: Val::Px(10.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                background_color: Color::rgba(0.1, 0.1, 0.1, 0.9).into(),
                ..default()
            },
            SettingsPanel,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn(TextBundle::from_section(
                "Post-Processing Controls",
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            spawn_effect_control(parent, "Exposure", EffectType::Exposure);
            spawn_effect_control(parent, "Bloom", EffectType::Bloom);
            spawn_effect_control(parent, "Chromatic Aberration", EffectType::ChromaticAberration);
            spawn_effect_control(parent, "Vignette", EffectType::Vignette);
            spawn_effect_control(parent, "Film Grain", EffectType::FilmGrain);
            spawn_effect_control(parent, "Lens Distortion", EffectType::LensDistortion);
            spawn_effect_control(parent, "Color Temperature", EffectType::ColorTemperature);

            // Tone mapping selector
            parent.spawn(TextBundle::from_section(
                "Tone Mapping",
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            let button_style = Style {
                padding: UiRect::all(Val::Px(5.0)),
                margin: UiRect::all(Val::Px(2.0)),
                ..default()
            };

            for (name, id) in [("None", 0), ("ACES", 1), ("Reinhard", 2), ("Uncharted2", 3)] {
                parent.spawn((
                    ButtonBundle {
                        style: button_style.clone(),
                        background_color: Color::rgb(0.2, 0.2, 0.2).into(),
                        ..default()
                    },
                    ToneMappingButton(id),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        name,
                        TextStyle {
                            font_size: 14.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                });
            }
        });
}

fn spawn_effect_control(builder: &mut ChildBuilder, label: &str, effect_type: EffectType) {
    builder
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        height: Val::Px(20.0),
                        margin: UiRect::left(Val::Px(10.0)),
                        ..default()
                    },
                    background_color: Color::rgb(0.2, 0.2, 0.2).into(),
                    ..default()
                },
                EffectSlider { effect_type },
            ));
        });
}

/// System to handle UI interactions
pub fn handle_ui_interactions(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &EffectSlider),
        (Changed<Interaction>, With<Button>),
    >,
    mut settings_query: Query<&mut PostProcessSettings>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let mut settings = settings_query.single_mut();

    for (interaction, mut color, slider) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if let Some(cursor_pos) = window.cursor_position() {
                    let value = (cursor_pos.x / 200.0).clamp(0.0, 1.0);
                    match slider.effect_type {
                        EffectType::Exposure => settings.exposure = value * 4.0 - 2.0, // Range: -2 to 2
                        EffectType::Bloom => settings.bloom_intensity = value,
                        EffectType::ChromaticAberration => settings.chromatic_aberration = value,
                        EffectType::Vignette => settings.vignette_strength = value,
                        EffectType::FilmGrain => settings.film_grain = value * 0.2, // Reduced range for subtlety
                        EffectType::LensDistortion => settings.lens_distortion = value * 0.4 - 0.2, // Range: -0.2 to 0.2
                        EffectType::ColorTemperature => settings.color_temperature = value,
                    }
                }
                *color = Color::rgb(0.3, 0.3, 0.3).into();
            }
            Interaction::Hovered => {
                *color = Color::rgb(0.25, 0.25, 0.25).into();
            }
            Interaction::None => {
                *color = Color::rgb(0.2, 0.2, 0.2).into();
            }
        }
    }
}

#[derive(Component)]
struct ToneMappingButton(u32);

/// System to handle tone mapping button clicks
pub fn handle_tone_mapping_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ToneMappingButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut settings_query: Query<&mut PostProcessSettings>,
) {
    for (interaction, mut color, button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let mut settings = settings_query.single_mut();
                settings.tone_mapping_type = button.0;
                *color = Color::rgb(0.3, 0.3, 0.3).into();
            }
            Interaction::Hovered => {
                *color = Color::rgb(0.25, 0.25, 0.25).into();
            }
            Interaction::None => {
                *color = Color::rgb(0.2, 0.2, 0.2).into();
            }
        }
    }
} 