use bevy::{prelude::*, render::pipelined_rendering::PipelinedRenderingPlugin, diagnostic::FrameTimeDiagnosticsPlugin, window::PresentMode};

fn main() {
    App::new()
        // Disable pipelined rendering to prioritize latency reduction
        // .add_plugins(DefaultPlugins.build().disable::<PipelinedRenderingPlugin>())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
          primary_window: Some(Window {
            title: "Bevytest".into(),
            resolution: (800., 600.).into(),
            present_mode: PresentMode::AutoVsync,
            fit_canvas_to_parent: true,
            prevent_default_event_handling: false,
            ..default()
          }),
          ..default()
        }))
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        .add_plugin(bevy_framepace::FramepacePlugin)
        .add_plugin(bevy_framepace::debug::DiagnosticsPlugin)
        .add_plugin(bevy_framepace::debug::CursorPlugin)
        .add_startup_system(setup)
        .add_system(toggle_plugin)
        .add_system(update_ui)
        .run();
}

#[derive(Component)]
struct EnableText;

fn toggle_plugin(
    mut settings: ResMut<bevy_framepace::FramepaceSettings>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        use bevy_framepace::Limiter;
        settings.limiter = match settings.limiter {
            Limiter::Auto => Limiter::Off,
            Limiter::Off => Limiter::from_framerate(30.0),
            Limiter::Manual(_) => Limiter::Auto,
        }
    }
}

fn update_ui(
    mut text: Query<&mut Text, With<EnableText>>,
    settings: Res<bevy_framepace::FramepaceSettings>,
) {
    text.single_mut().sections[1].value = format!("{}", settings.limiter);
}

/// set up the scene
fn setup(mut commands: Commands, mut windows: Query<&mut Window>, asset_server: Res<AssetServer>) {
    windows.iter_mut().next().unwrap().cursor.icon = CursorIcon::Crosshair;
    commands.spawn((Camera3dBundle::default(),));
    // UI
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    let style = TextStyle {
        font,
        font_size: 60.0,
        color: Color::WHITE,
    };
    commands.spawn((
        TextBundle::from_sections(vec![
            TextSection {
                value: "Frame pacing: ".to_string(),
                style: style.clone(),
            },
            TextSection {
                value: "".to_string(),
                style: style.clone(),
            },
            TextSection {
                value: "\n[press space]".to_string(),
                style,
            },
        ]),
        EnableText,
    ));
}