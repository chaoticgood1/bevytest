use bevy::{prelude::*, window::PresentMode, render::{render_resource::{PrimitiveTopology, VertexFormat}, mesh::{Indices, MeshVertexAttribute}}, window::{PrimaryWindow, CursorGrabMode}};
use bevy_egui::{EguiContexts, egui::{self, TextureId, Frame, Color32, Style, ImageButton, Rect, Vec2, Pos2, RichText}, EguiPlugin};
use bevy_flycam::FlyCam;
use bevy_flycam::NoCameraPlayerPlugin;
use noise::*;

fn main() {
  let mut app = App::new();
  app
    .add_plugins(DefaultPlugins.set(WindowPlugin {
      primary_window: Some(Window {
        title: "Ironverse Editor".into(),
        resolution: (800., 600.).into(),
        present_mode: PresentMode::AutoVsync,
        fit_canvas_to_parent: true,
        prevent_default_event_handling: false,
        ..default()
      }),
      ..default()
    }))
    .add_plugin(NoCameraPlayerPlugin);

  app
    .add_plugin(EguiPlugin)
    .add_startup_system(setup_camera)
    .add_startup_system(test_fast_surface_net)
    .add_system(update);

  app.run();
}



fn setup_camera(
  mut commands: Commands,
) {
  commands
    .spawn(Camera3dBundle {
      transform: Transform::from_xyz(-8.0, 7.0, -4.0)
        .looking_to(Vec3::new(0.7, -0.3, 0.6), Vec3::Y),
      ..Default::default()
    })
    .insert(FlyCam);

  commands.spawn(PointLightBundle {
    point_light: PointLight {
      intensity: 6000.0,
      ..Default::default()
    },
    transform: Transform::from_xyz(0.0, 8.0, 8.0),
    ..Default::default()
  });
}

fn test_fast_surface_net(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {

  let noise = OpenSimplex::new().set_seed(1234);

  use fast_surface_nets::ndshape::{ConstShape, ConstShape3u32};
  use fast_surface_nets::{surface_nets, SurfaceNetsBuffer};

  // A 16^3 chunk with 1-voxel boundary padding.
  type ChunkShape = ConstShape3u32<18, 18, 18>;

  // This chunk will cover just a single octant of a sphere SDF (radius 15).
  let mut sdf = [1.0; ChunkShape::USIZE];
  for i in 0u32..ChunkShape::SIZE {
    let [x, y, z] = ChunkShape::delinearize(i);

    let elevation = elevation(&x, &z, &0, noise);
    let mid = y as i64 - 4;
    // info!("elevation {:?}", elevation);
    if elevation > mid {
      sdf[i as usize] = -1.0;
    }

    if x == 5 && y == 5 && z == 10 {
      sdf[i as usize] = -1.0;
    }

    if x == 2 && y == 3 && z == 10 {
      sdf[i as usize] = 1.0;
    }
  }

  let mut buffer = SurfaceNetsBuffer::default();
  surface_nets(&sdf, &ChunkShape {}, [0; 3], [17; 3], &mut buffer);


  let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
  render_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, buffer.positions);
  render_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, buffer.normals);
  render_mesh.set_indices(Some(Indices::U32(buffer.indices)));

  let mesh_handle = meshes.add(render_mesh);

  // let coord_f32 = key_to_world_coord_f32(&[0, 0, 0], manager.config.seamless_size);
  let coord_f32 = [0.0, 0.0, 0.0];
  commands
    .spawn(MaterialMeshBundle {
      mesh: mesh_handle,
      material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
      transform: Transform::from_xyz(coord_f32[0], coord_f32[1], coord_f32[2]),
      ..default()
    });
}

fn elevation(x: &u32, z: &u32, middle: &i64, noise: OpenSimplex) -> i64 {
  let frequency = 0.0125;
  // let frequency = 0.05;
  let height_scale = 16.0;
  let fx = (*x as i64 - middle) as f64 * frequency;
  let fz = (*z as i64 - middle) as f64 * frequency;
  let noise = noise.get([fx, fz]);
  let elevation = (noise * height_scale) as i64;
  elevation
}


fn update(
  cameras: Query<&Transform, With<FlyCam>>,
  mut ctx: EguiContexts,
  mut windows: Query<&mut Window, With<PrimaryWindow>>,
  key_input: Res<Input<KeyCode>>,
) {
  let mut window = match windows.get_single_mut() {
    Ok(w) => { w },
    Err(_e) => return,
  };

  if key_input.just_pressed(KeyCode::LControl) {
    match window.cursor.grab_mode {
      CursorGrabMode::None => {
        window.cursor.grab_mode = CursorGrabMode::Confined;
        window.cursor.visible = false;
      }
      _ => {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
      }
    }
  }
  

  let frame = Frame {
    fill: Color32::from_rgba_unmultiplied(0, 0, 0, 0),
    ..Default::default()
  };

  egui::Window::new("ChunkTexts")
    .title_bar(false)
    .frame(frame)
    .fixed_rect(Rect::from_min_max(
      Pos2::new(0.0, 0.0),
      Pos2::new(window.width(), window.height())
    ))
    .show(ctx.ctx_mut(), |ui| {
      ui.vertical(|ui| {
        let mut style = Style::default();
        style.spacing.item_spacing = Vec2::new(0.0, 0.0);
        ui.set_style(style);

        for trans in &cameras {
          ui.label(
            RichText::new(format!("Pos: {:?}", trans.translation))
              .color(Color32::WHITE)
              .size(20.0)
          );

          ui.label(
            RichText::new(format!("Forward: {:?}", trans.forward()))
              .color(Color32::WHITE)
              .size(20.0)
          );
        }
      });
    });

  
}


