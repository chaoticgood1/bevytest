use bevy::{prelude::*, render::{mesh::{MeshVertexAttribute, MeshVertexBufferLayout, Indices}, render_resource::{VertexFormat, AsBindGroup, ShaderRef, SpecializedMeshPipelineError, RenderPipelineDescriptor, PrimitiveTopology}}, reflect::TypeUuid, pbr::{MaterialPipeline, MaterialPipelineKey}, asset::LoadState};
use bevy_flycam::prelude::*;

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(PlayerPlugin)
    .add_plugin(MaterialPlugin::<CustomMaterial>::default())
    .add_startup_system(startup)
    .add_system(init_textures)
    .run();
}



fn startup(
  mut commands: Commands, 
  asset_server: Res<AssetServer>,
) {
  commands.insert_resource(ChunkTexture {
    is_loaded: false,
    albedo: asset_server.load("textures/terrains_albedo.png"),
    normal: asset_server.load("textures/terrains_normal.png"),
  });

  // light
  commands.spawn(PointLightBundle {
    point_light: PointLight {
      intensity: 3000.0,
      ..Default::default()
    },
    transform: Transform::from_xyz(-3.0, 2.0, -1.0),
    ..Default::default()
  });
  commands.spawn(PointLightBundle {
    point_light: PointLight {
      intensity: 3000.0,
      ..Default::default()
    },
    transform: Transform::from_xyz(3.0, 2.0, 1.0),
    ..Default::default()
  });

  // // camera
  // commands.spawn(Camera3dBundle {
  //   transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::new(1.5, 0.0, 0.0), Vec3::Y),
  //   ..Default::default()
  // });
}
  
fn init_textures(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut custom_materials: ResMut<Assets<CustomMaterial>>,
  mut _materials: ResMut<Assets<StandardMaterial>>,
  asset_server: Res<AssetServer>,

  mut loading_texture: ResMut<ChunkTexture>,
  mut images: ResMut<Assets<Image>>,
) {
  if loading_texture.is_loaded
    || asset_server.get_load_state(loading_texture.albedo.clone()) != LoadState::Loaded
    || asset_server.get_load_state(loading_texture.normal.clone()) != LoadState::Loaded
  {
    return;
  }
  loading_texture.is_loaded = true;

  let array_layers = 4;
  let image = images.get_mut(&loading_texture.albedo).unwrap();
  image.reinterpret_stacked_2d_as_array(array_layers);

  let normal = images.get_mut(&loading_texture.normal).unwrap();
  normal.reinterpret_stacked_2d_as_array(array_layers);


  let render_mesh = Mesh::from(shape::Cube { size: 1.0 });
  let mesh_handle = meshes.add(render_mesh);
  let material_handle = custom_materials.add(CustomMaterial {
    albedo: loading_texture.albedo.clone(),
    normal: loading_texture.normal.clone(),
  });

  commands
    .spawn(MaterialMeshBundle {
      mesh: mesh_handle,
      material: material_handle,
      transform: Transform::from_xyz(0.0, 0.0, 0.0),
      ..default()
    });
}

#[derive(Resource)]
struct ChunkTexture {
  is_loaded: bool,
  albedo: Handle<Image>,
  normal: Handle<Image>,
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "5f2e1d29-b8ad-4680-8c96-f8b78a580718"]
struct CustomMaterial {
  #[texture(0, dimension = "2d_array")]
  #[sampler(1)]
  albedo: Handle<Image>,
  #[texture(2, dimension = "2d_array")]
  #[sampler(3)]
  normal: Handle<Image>,
}

impl Material for CustomMaterial {
  fn vertex_shader() -> ShaderRef {
    "shaders/triplanar.wgsl".into()
  }
  fn fragment_shader() -> ShaderRef {
    "shaders/triplanar.wgsl".into()
  }
  fn specialize(
    _pipeline: &MaterialPipeline<Self>,
    descriptor: &mut RenderPipelineDescriptor,
    layout: &MeshVertexBufferLayout,
    _key: MaterialPipelineKey<Self>,
  ) -> Result<(), SpecializedMeshPipelineError> {
    let vertex_layout = layout.get_layout(&[
      Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
      Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
    ])?;
    descriptor.vertex.buffers = vec![vertex_layout];

    Ok(())
  }
}
  