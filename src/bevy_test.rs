use bevy::{
  prelude::*,
  render::{
    pipeline::{DynamicBinding, PrimitiveTopology, PipelineDescriptor, 
      PipelineSpecialization, RenderPipeline
    },
    render_graph::{base, AssetRenderResourcesNode, RenderGraph},
    renderer::RenderResources,
    shader::{asset_shader_defs_system, ShaderDefs, ShaderStage, ShaderStages},
  },
  type_registry::TypeUuid,
};
use bevy::render::mesh::Indices;

// use bevy_mod_picking::*;
use bevy_rapier3d::rapier::{dynamics::RigidBodyBuilder, geometry::ColliderBuilder, math::Point};
use building_blocks::{mesh::surface_nets::*, prelude::*, storage::chunk_map::ChunkMap};
use noise::*;

use std::collections::HashMap;
use std::collections::HashSet;


pub struct CustomPlugin;
impl Plugin for CustomPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .init_resource::<TerrainResource>()
      .add_plugins(DefaultPlugins)
      .add_asset::<MyMaterial>()
      .add_startup_system(startup.system())
      .add_system_to_stage(
          stage::POST_UPDATE,
          asset_shader_defs_system::<MyMaterial>.system(),
      )
      .run();
  }
}

#[derive(RenderResources, ShaderDefs, Default, TypeUuid)]
#[uuid = "620f651b-adbe-464b-b740-ba0e547282ba"]
struct MyMaterial {
    pub albedo: Color,
    #[shader_def]
    pub albedo_texture: Option<Handle<Texture>>,
    #[render_resources(ignore)]
    #[shader_def]
    pub shaded: bool,
}

const VERTEX_SHADER: &str = r#"
#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in vec2 Vertex_Uv;

layout(location = 0) out vec3 v_Position;
layout(location = 1) out vec3 v_Normal;
layout(location = 2) out vec2 v_Uv;

layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};

layout(set = 2, binding = 0) uniform Transform {
    mat4 Model;
};

void main() {
    v_Normal = (Model * vec4(Vertex_Normal, 1.0)).xyz;
    v_Normal = mat3(Model) * Vertex_Normal;
    v_Position = (Model * vec4(Vertex_Position, 1.0)).xyz;
    v_Uv = Vertex_Uv;
    gl_Position = ViewProj * vec4(v_Position, 1.0);
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 450

const int MAX_LIGHTS = 10;

struct Light {
    mat4 proj;
    vec4 pos;
    vec4 color;
};

layout(location = 0) in vec3 v_Position;
layout(location = 1) in vec3 v_Normal;
layout(location = 2) in vec2 v_Uv;

layout(location = 0) out vec4 o_Target;

layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};

layout(set = 1, binding = 0) uniform Lights {
    uvec4 NumLights;
    Light SceneLights[MAX_LIGHTS];
};

layout(set = 3, binding = 0) uniform MyMaterial_albedo {
    vec4 Albedo;
};

# ifdef MYMATERIAL_ALBEDO_TEXTURE
layout(set = 3, binding = 1) uniform texture2D MyMaterial_albedo_texture;
layout(set = 3, binding = 2) uniform sampler MyMaterial_albedo_texture_sampler;
# endif

void main() {
    vec4 output_color = Albedo;
# ifdef MYMATERIAL_ALBEDO_TEXTURE
    output_color *= texture(
        sampler2D(MyMaterial_albedo_texture, MyMaterial_albedo_texture_sampler),
        v_Uv);
# endif

# ifdef MYMATERIAL_SHADED
    vec3 normal = normalize(v_Normal);
    vec3 ambient = vec3(0.05, 0.05, 0.05);
    // accumulate color
    vec3 color = ambient;
    for (int i=0; i<int(NumLights.x) && i<MAX_LIGHTS; ++i) {
        Light light = SceneLights[i];
        // compute Lambertian diffuse term
        vec3 light_dir = normalize(light.pos.xyz - v_Position);
        float diffuse = max(0.0, dot(normal, light_dir));
        // add light contribution
        color += diffuse * light.color.xyz;
    }
    output_color.xyz *= color;
# endif

    // multiply the light by material color
    o_Target = output_color;
}

"#;

pub struct AdjacentPoint {
  pub point: PointN<[i32; 3]>,
  pub distance: f32,
}

pub struct TerrainResource {
  pub chunk_shape: PointN<[i32; 3]>,
  pub map: ChunkMap<[i32; 3], f32>,
  pub map_chunk_graphics: HashMap<PointN<[i32; 3]>, Option<Entity>>,
  pub mutated_chunk_keys: Vec<PointN<[i32; 3]>>,
  pub noise: OpenSimplex,

  // Render
  pub selected_voxel_pos: Option<Vec3>,
  pub new_voxel_pos: Option<Vec3>,
  pub refresh_picking: bool,
}

impl Default for TerrainResource {
  fn default() -> Self {
    let chunk_shape = PointN([16; 3]);
    TerrainResource {
      chunk_shape: chunk_shape,
      map: ChunkMap3::new(chunk_shape, 1.0, (), FastLz4 { level: 10 }),
      map_chunk_graphics: HashMap::new(),
      mutated_chunk_keys: Vec::new(),
      noise: OpenSimplex::new().set_seed(1234),
      selected_voxel_pos: None,
      new_voxel_pos: None,
      refresh_picking: true,
    }
  }
}

impl TerrainResource {
  pub fn new_terrain_location(self: &mut Self, cursor_pos: &Vec3) -> Option<PointN<[i32; 3]>> {
    let adj_pts = self.adjacent_points(cursor_pos);
    for adj in adj_pts.iter() {
      let val = self.map.get_mut(&adj.point);
      if val.clone() > -1.0 {
        return Some(adj.point);
      }
    }
    None
  }

  fn adjacent_points(self: &mut Self, cursor_pos: &Vec3) -> Vec<AdjacentPoint> {
    let mut points: Vec<AdjacentPoint> = Vec::new();
    let voxel_point = Vec3::new(
      cursor_pos.x().floor(),
      cursor_pos.y().floor(),
      cursor_pos.z().floor(),
    );

    // Get all the surrounding points by 1 unit
    for x in -1..2 {
      for y in -1..2 {
        for z in -1..2 {
          if x == 0 && y == 0 && z == 0 {
            // The point itself, don't process
            continue;
          }

          let local_point = PointN([
            voxel_point.x() as i32 + x,
            voxel_point.y() as i32 + y,
            voxel_point.z() as i32 + z,
          ]);
          let diff = cursor_pos.clone()
            - Vec3::new(
              local_point.0[0] as f32 + 0.5,
              local_point.0[1] as f32 + 0.5,
              local_point.0[2] as f32 + 0.5,
            );
          points.push(AdjacentPoint {
            point: local_point,
            distance: diff.length_squared(),
          });
        }
      }
    }
    points.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    points
  }

  pub fn pos_val(self: &mut Self, point: Vec3) -> f32 {
    let point_n = PointN([point.x() as i32, point.y() as i32, point.z() as i32]);
    return self.map.get_mut(&point_n).clone();
  }

  pub fn init_mutated_chunk_keys(self: &mut Self, point: PointN<[i32; 3]>, clear_cache: bool) {
    if clear_cache {
      self.mutated_chunk_keys = Vec::new();
    }

    let chunk_key = self.map.chunk_key(&point);
    if !self
      .mutated_chunk_keys
      .iter()
      .any(|point| point.clone() == chunk_key)
    {
      self.mutated_chunk_keys.push(chunk_key);
    }
  }

  pub fn points_around(point: PointN<[i32; 3]>) -> Vec<PointN<[i32; 3]>> {
    let mut points: Vec<PointN<[i32; 3]>> = Vec::new();
    for x in -1..2 {
      for y in -1..2 {
        for z in -1..2 {
          if x == 0 && y == 0 && z == 0 {
            // The point itself, don't process
            continue;
          }

          let local_point = PointN([point.0[0] + x, point.0[1] + y, point.0[2] + z]);
          points.push(local_point);
        }
      }
    }
    points
  }
}

fn startup(
  mut commands: &mut Commands,
  mut pipelines: ResMut<Assets<PipelineDescriptor>>,
  mut shaders: ResMut<Assets<Shader>>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<MyMaterial>>,
  mut render_graph: ResMut<RenderGraph>,
  asset_server: Res<AssetServer>,

  mut terrain_resource: ResMut<TerrainResource>,
) {
  commands
    .spawn(LightComponents {
      transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
      ..Default::default()
    })
    .spawn(Camera3dComponents {
      transform: Transform::from_translation(Vec3::new(3.0, 5.0, -8.0))
          .looking_at(Vec3::default(), Vec3::unit_y()),
      ..Default::default()
    });

  // Create terrain
  // This will create terrain around the center
  let radius = 20; // Should be per chunk
  let height_scale = 30.0;
  let frequency = 0.025;
  let pos_x = 0;
  let pos_z = 0;
  let start_x = pos_x - radius;
  let start_z = pos_z - radius;
  for x in start_x..radius {
    for z in start_z..radius {
      let fx = x as f64 * frequency;
      let fz = z as f64 * frequency;
      let noise_y = terrain_resource.noise.get([fx, fz]);
      let y = noise_y * height_scale;
      let point = PointN([x as i32, y as i32, z as i32]);
      *terrain_resource.map.get_mut(&point) = -1.0;
      terrain_resource.init_mutated_chunk_keys(point, false);
    }
  }

  generate_meshes(
    &mut commands,
    &mut meshes,
    &mut materials,
    &mut terrain_resource,
    &mut render_graph,
    &asset_server,
    &mut shaders,
    &mut pipelines,
  );
}

fn generate_meshes(
  commands: &mut Commands,
  meshes: &mut ResMut<Assets<Mesh>>,
  materials: &mut ResMut<Assets<MyMaterial>>,
  terrain_resource: &mut ResMut<TerrainResource>,
  render_graph: &mut ResMut<RenderGraph>,
  asset_server: &Res<AssetServer>,
  shaders: &mut ResMut<Assets<Shader>>,
  pipelines: &mut ResMut<Assets<PipelineDescriptor>>,
) {
  let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
    vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
    fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
  }));

  // Add an AssetRenderResourcesNode to our Render Graph. This will bind MyMaterial resources to our shader
  render_graph.add_system_node(
    "my_material",
    AssetRenderResourcesNode::<MyMaterial>::new(true),
  );

  // Add a Render Graph edge connecting our new "my_material" node to the main pass node. This ensures "my_material" runs before the main pass
  render_graph
      .add_node_edge("my_material", base::node::MAIN_PASS)
      .unwrap();

  // let texture_handle = asset_server.load("albedo.png");
  let texture_handle = asset_server.load("elon.png");

  // Create a green material
  let green_material: Handle<MyMaterial> = materials.add(MyMaterial {
    albedo: Color::rgb(0.0, 0.8, 0.0),
    albedo_texture: Some(texture_handle.clone()),
    shaded: true,
  });

  // Create a cube mesh which will use our materials
  let cube_handle = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

  commands
    // cube
    .spawn(MeshComponents {
        mesh: cube_handle.clone(),
        render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::specialized(
            pipeline_handle.clone(),
            // NOTE: in the future you wont need to manually declare dynamic bindings
            PipelineSpecialization {
                dynamic_bindings: vec![
                    // Transform
                    DynamicBinding {
                      bind_group: 2,
                      binding: 0,
                    },
                    // // MyMaterial_color
                    // DynamicBinding {
                    //     bind_group: 1,
                    //     binding: 1,
                    // },
                    DynamicBinding {
                        bind_group: 3,
                        binding: 0,
                    },
                    // // MyMaterial_color
                    // DynamicBinding {
                    //     bind_group: 3,
                    //     binding: 1,
                    // },
                ],
                ..Default::default()
            },
        )]),
        transform: Transform::from_translation(Vec3::new(-2.0, 0.0, 0.0)),
        ..Default::default()
    })
    .with(green_material);







  // let mutated_chunk_keys = &terrain_resource.mutated_chunk_keys;
  // let map = &terrain_resource.map;
  // // let map_chunk_graphics = &terrain_resource.map_chunk_graphics;
  // let chunk_shape = terrain_resource.chunk_shape.clone();

  // // For each mutated chunk, and any adjacent chunk, the mesh will need to be updated. (This is
  // // not 100% true, but it's a conservative assumption to make. In reality, if a given voxel is
  // // mutated and it's L1 distance from an adjacent chunk is less than or equal to 2, that adjacent
  // // chunk's mesh is dependent on that voxel, so it must be re-meshed).
  // let mut chunk_keys_to_update: HashSet<Point3i> = HashSet::new();
  // let offsets = Point3i::moore_offsets();
  // for chunk_key in mutated_chunk_keys.iter() {
  //   chunk_keys_to_update.insert(*chunk_key);
  //   for offset in offsets.iter() {
  //     chunk_keys_to_update.insert(*chunk_key + *offset * chunk_shape);
  //   }
  // }

  // // Now we generate mesh vertices for each chunk.
  // let local_cache = LocalChunkCache::new();
  // let reader = ChunkMapReader3::new(map, &local_cache);
  // for chunk_key in chunk_keys_to_update.into_iter() {
  //   let padded_chunk_extent = map.extent_for_chunk_at_key(&chunk_key).padded(1);
  //   let mut padded_chunk = Array3::fill(padded_chunk_extent, 0.0);
  //   copy_extent(&padded_chunk_extent, &reader, &mut padded_chunk);
  //   let mut sn_buffer = SurfaceNetsBuffer::new(padded_chunk_extent.num_points());
  //   surface_nets(&padded_chunk, &padded_chunk_extent, &mut sn_buffer);

  //   let mut indices_u32: Vec<u32> = vec![];
  //   for i in sn_buffer.indices {
  //     indices_u32.push(i as u32);
  //   }
  //   let indices_points = convert_flatu32_to_points(indices_u32.clone());

  //   if indices_u32.len() == 0 {
  //     continue;
  //   }

  //   let positions = sn_buffer.positions.clone();
  //   let normals = sn_buffer.normals.clone();
  //   let mesh = create_mesh(meshes, positions, normals, indices_u32);

  //   // Create as a function
  //   let body = RigidBodyBuilder::new_static().translation(0., 0., 0.);
  //   let collider = ColliderBuilder::trimesh(
  //     convert_arrayf32_to_points(sn_buffer.positions),
  //     indices_points,
  //   );

  //   commands
  //   // cube
  //   .spawn(MeshComponents {
  //     mesh: mesh,
  //     render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::specialized(
  //       pipeline_handle.clone(),
  //       // NOTE: in the future you wont need to manually declare dynamic bindings
  //       PipelineSpecialization {
  //         dynamic_bindings: vec![
  //           // DynamicBinding {
  //           //   bind_group: 0,
  //           //   binding: 0,
  //           // },
  //           // DynamicBinding {
  //           //   bind_group: 1,
  //           //   binding: 1,
  //           // },
  //           // Transform
  //           DynamicBinding {
  //             bind_group: 2,
  //             binding: 0,
  //           },
  //           // MyMaterial_color
  //           DynamicBinding {
  //             bind_group: 3,
  //             binding: 0,
  //           },
  //         ],
  //         ..Default::default()
  //       },
  //     )]),
  //     transform: Transform::from_translation(Vec3::new(-2.0, 0.0, 0.0)),
  //     ..Default::default()
  //   })
  //   .with(green_material.clone())
  //   .with(body)
  //   .with(collider)
  //   ;
  // }
}

fn create_mesh(
  meshes: &mut ResMut<Assets<Mesh>>,
  positions: Vec<[f32; 3]>,
  normals: Vec<[f32; 3]>,
  indices: Vec<u32>,
) -> Handle<Mesh> {
  let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
  mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions.into());
  mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals.into());
  // mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, [].into());
  mesh.set_indices(Some(Indices::U32(indices)));
  meshes.add(mesh)
}

fn convert_flatu32_to_points(array: Vec<u32>) -> Vec<Point<u32>> {
  let mut points: Vec<Point<u32>> = Vec::new();

  let slices: Vec<&[u32]> = array.chunks(3).collect();
  for slice in slices.iter() {
    points.push(Point::new(slice[0], slice[1], slice[2]));
  }
  points
}

fn convert_arrayf32_to_points(array: Vec<[f32; 3]>) -> Vec<Point<f32>> {
  let mut points: Vec<Point<f32>> = Vec::new();

  for array_p in array.iter() {
    points.push(Point::new(array_p[0], array_p[1], array_p[2]));
  }
  points
}

