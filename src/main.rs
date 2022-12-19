/* use bevy::{
  ecs::system::{lifetimeless::SRes, SystemParamItem},
  pbr::{MaterialPipeline, SpecializedMaterial},
  prelude::*,
  reflect::TypeUuid,
  render::{
    mesh::{MeshVertexBufferLayout, Indices, MeshVertexAttribute},
    render_asset::{PrepareAssetError, RenderAsset, RenderAssets},
    render_resource::{
      std140::{AsStd140, Std140},
      *,
    },
    renderer::RenderDevice,
  },
};
use voxels::data::{voxel_octree::{VoxelOctree, ParentValueType, VoxelMode}, surface_nets::VoxelReuse};

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(MaterialPlugin::<CustomMaterial>::default())
    .add_startup_system(setup)
    .run();
}


/*
  Implement getting the mesh from the surface nets
  Then start the triplanar

  Alternatives:
    Render cube
    Basic texturing
    Then do triplanar
*/

fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<CustomMaterial>>,
  mut standard_materials: ResMut<Assets<StandardMaterial>>,
  asset_server: Res<AssetServer>,
) {

  let mut octree = VoxelOctree::new_from_3d_array(
    0, 4,
    &vec![
      [2, 2, 2, 1],
      [2, 3, 2, 1],
      [2, 2, 3, 1],
    ].to_vec(),
    ParentValueType::DefaultValue,
  );

  let data = octree.compute_mesh2(VoxelMode::SurfaceNets, &mut VoxelReuse::new(4, 3));
  

  let mut layers = Vec::new();
  for i in 0..data.positions.len() {
    layers.push(1);
  }
  

  // println!("{:?}", layers);
  // println!("layer {:?}", layers.len());
  println!("normals {:?}", data.normals.len());
  println!("positions {:?}", data.positions.len());
  println!("data.uvs {:?}", data.uvs.len());
  println!("indices {:?}", data.indices.len());
  
  let attr = MeshVertexAttribute::new("Vertex_Layer", 7, VertexFormat::Uint32);
  
  let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
  render_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.positions);
  render_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, data.normals);
  render_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, data.uvs);
  render_mesh.insert_attribute(attr, layers);
  render_mesh.set_indices(Some(Indices::U32(data.indices)));

  // commands
  //   .spawn_bundle(MaterialMeshBundle {
  //     mesh: meshes.add(render_mesh),
  //     transform: Transform::from_xyz(-2.0, -2.0, -2.0),
  //     material: materials.add(CustomMaterial {
  //       texture: asset_server.load(
  //           "materials.png",
  //       ),
  //   }),
  //     ..default()
  //   });

    // .spawn_bundle(PbrBundle {
    //   mesh: meshes.add(render_mesh),
    //   material: standard_materials.add(Color::rgba(1.0, 1.0, 1.0, 1.0).into()),
    //   transform: Transform::from_xyz(0.0, 0.5, 0.0),
    //   ..Default::default()
    // });

    // .spawn_bundle(PbrBundle {
    //   mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //   material: standard_materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
    //   transform: Transform::from_xyz(0.0, 0.5, 0.0),
    //   ..default()
    // });


  commands.spawn().insert_bundle(MaterialMeshBundle {
    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    transform: Transform::from_xyz(0.0, 0.5, 0.0),
    material: materials.add(CustomMaterial {
        color: Color::GREEN,
    }),
    ..default()
  });

  // camera
  commands.spawn_bundle(PerspectiveCameraBundle {
    transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ..default()
  });
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "4ee9c363-1124-4113-890e-199d81b00281"]
pub struct CustomMaterial {
    color: Color,
}

#[derive(Clone)]
pub struct GpuCustomMaterial {
    _buffer: Buffer,
    bind_group: BindGroup,
}

impl RenderAsset for CustomMaterial {
    type ExtractedAsset = CustomMaterial;
    type PreparedAsset = GpuCustomMaterial;
    type Param = (SRes<RenderDevice>, SRes<MaterialPipeline<Self>>);
    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, material_pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let color = Vec4::from_slice(&extracted_asset.color.as_linear_rgba_f32());
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: color.as_std140().as_bytes(),
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
            layout: &material_pipeline.material_layout,
        });

        Ok(GpuCustomMaterial {
            _buffer: buffer,
            bind_group,
        })
    }
}

impl SpecializedMaterial for CustomMaterial {
    type Key = ();

    fn key(_: &<CustomMaterial as RenderAsset>::PreparedAsset) -> Self::Key {}

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _: Self::Key,
        _layout: &MeshVertexBufferLayout,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.entry_point = "main".into();
        descriptor.fragment.as_mut().unwrap().entry_point = "main".into();
        Ok(())
    }

    fn vertex_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/custom_material.vert"))
    }

    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/custom_material.frag"))
    }

    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(Vec4::std140_size_static() as u64),
                },
                count: None,
            }],
            label: None,
        })
    }
}
 */




use bevy::{
  asset::LoadState,
  prelude::*,
  reflect::TypeUuid,
  render::{render_resource::{AsBindGroup, ShaderRef, PrimitiveTopology}, mesh::Indices},
};
use voxels::data::{voxel_octree::{VoxelOctree, ParentValueType, VoxelMode}, surface_nets::VoxelReuse};
use smooth_bevy_cameras::{
  controllers::{unreal::{UnrealCameraBundle, UnrealCameraController, UnrealCameraPlugin}, fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin}},
  LookTransformPlugin,
};

/// This example illustrates how to create a texture for use with a `texture_2d_array<f32>` shader
/// uniform variable.
fn main() {
  App::new()
      .add_plugins(DefaultPlugins)
      .add_plugin(MaterialPlugin::<ArrayTextureMaterial>::default())
      .add_plugin(LookTransformPlugin)
      .add_plugin(UnrealCameraPlugin::default())
      .add_plugin(FpsCameraPlugin::default())
      .add_startup_system(setup)
      .add_system(create_array_texture)
      .run();
}

#[derive(Resource)]
struct LoadingTexture {
  is_loaded: bool,
  handle: Handle<Image>,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
  // Start loading the texture.
  commands.insert_resource(LoadingTexture {
      is_loaded: false,
      handle: asset_server.load("textures/array_texture.png"),
  });

  // // light
  // commands.spawn(PointLightBundle {
  //     point_light: PointLight {
  //         intensity: 3000.0,
  //         ..Default::default()
  //     },
  //     transform: Transform::from_xyz(-3.0, 2.0, -1.0),
  //     ..Default::default()
  // });
  // commands.spawn(PointLightBundle {
  //     point_light: PointLight {
  //         intensity: 3000.0,
  //         ..Default::default()
  //     },
  //     transform: Transform::from_xyz(3.0, 2.0, 1.0),
  //     ..Default::default()
  // });
  
  use std::f32::consts::PI;
  commands.spawn(DirectionalLightBundle {
    directional_light: DirectionalLight {
      illuminance: 32000.0,
      ..default()
    },
    transform: Transform::from_xyz(0.0, 2.0, 0.0)
      .with_rotation(Quat::from_rotation_x(-PI / 4.)),
      ..default()
  });

  // commands.insert_resource(AmbientLight {
  //   color: Color::rgb_u8(255, 255, 255),
  //   brightness: 1.0,
  // });

  // commands
  //   .spawn(Camera3dBundle::default())
  //   .insert(FpsCameraBundle::new(
  //     FpsCameraController::default(),
  //     Vec3::new(-2.0, 5.0, 5.0),
  //     Vec3::new(0., 0., 0.),
  //   ));

  commands
    .spawn(Camera3dBundle::default())
    .insert(UnrealCameraBundle::new(
        UnrealCameraController::default(),
        Vec3::new(-2.0, 5.0, 5.0),
        Vec3::new(0., 0., 0.),
    ));
}

fn create_array_texture(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut loading_texture: ResMut<LoadingTexture>,
  mut images: ResMut<Assets<Image>>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ArrayTextureMaterial>>,
) {
  if loading_texture.is_loaded
    || asset_server.get_load_state(loading_texture.handle.clone()) != LoadState::Loaded
  {
    return;
  }
  loading_texture.is_loaded = true;
  let image = images.get_mut(&loading_texture.handle).unwrap();

  // Create a new array texture asset from the loaded texture.
  let array_layers = 4;
  image.reinterpret_stacked_2d_as_array(array_layers);



  
  let mut octree = VoxelOctree::new_from_3d_array(
    0, 4,
    &vec![
      [2, 2, 2, 1],
      [2, 3, 2, 1],
      // [2, 2, 3, 1],
    ].to_vec(),
    ParentValueType::DefaultValue,
  );

  let data = octree.compute_mesh2(VoxelMode::SurfaceNets, &mut VoxelReuse::new(4, 3));
  let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
  render_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.positions);
  render_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, data.normals);
  render_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, data.uvs);
  // render_mesh.insert_attribute(attr, layers);
  render_mesh.set_indices(Some(Indices::U32(data.indices)));


  // Spawn some cubes using the array texture
  // let mesh_handle = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
  // let mesh_handle = meshes.add(render_mesh);
  let mesh_handle = meshes.add(Mesh::from(shape::Icosphere { radius: 1.0, subdivisions: 12 }));
  

  // let mut layers = Vec::new();
  // for i in 0..data.positions.len() {
  //   layers.push(1);
  // }



  let material_handle = materials.add(ArrayTextureMaterial {
    array_texture: loading_texture.handle.clone(),
  });
  // for x in -5..=5 {
  //   // println!("x {}", x);
  //   commands.spawn(MaterialMeshBundle {
  //     mesh: mesh_handle.clone(),
  //     material: material_handle.clone(),
  //     transform: Transform::from_xyz(x as f32 * 2.0 + 0.5, 0.0, 0.0),
  //     ..Default::default()
  //   });
  // }

  commands.spawn(MaterialMeshBundle {
    mesh: mesh_handle.clone(),
    material: material_handle.clone(),
    transform: Transform::from_xyz(0 as f32 * 2.0 + 0.5, 0.0, 0.0),
    ..Default::default()
  });
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "9c5a0ddf-1eaf-41b4-9832-ed736fd26af3"]
struct ArrayTextureMaterial {
  #[texture(0, dimension = "2d_array")]
  #[sampler(1)]
  array_texture: Handle<Image>,
}

impl Material for ArrayTextureMaterial {
  fn fragment_shader() -> ShaderRef {
    "shaders/array_texture.wgsl".into()
  }
}
