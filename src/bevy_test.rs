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

layout(set = 0, binding = 0) uniform Camera {
  mat4 ViewProj;
};

// Copied from amethyst_rendy
layout(location = 0) in vec3 position;
layout(location = 1) in vec4 material_weights;
layout(location = 2) in vec3 normal;
layout(location = 3) in mat4 model; // instance rate
layout(location = 7) in vec4 tint; // instance rate

layout(location = 0) out vec3 out_position;
layout(location = 1) out vec3 out_normal;
layout(location = 2) out vec4 out_color;
layout(location = 3) out vec4 out_material_weights;

void main() {
    vec4 vertex_position = model * vec4(position, 1.0);
    out_position = vertex_position.xyz;
    out_normal = normalize(mat3(model) * normal);
    // out_color = tint;
    out_color = vec4(1.0, 1.0, 1.0, 1.0);
    out_material_weights = vec4(1.0, 1.0, 1.0, 1.0);
    // out_material_weights = material_weights / dot(material_weights, vec4(1.0));
    gl_Position = ViewProj * vertex_position;
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 450

// Copied from amethyst_rendy, augmented for triplanar mapping

const float PI = 3.14159265359;

struct UvOffset {
    vec2 u_offset;
    vec2 v_offset;
};

float tex_coord(float coord, vec2 offset) {
    return offset.x + coord * (offset.y - offset.x);
}

vec2 tex_coords(vec2 coord, UvOffset offset) {
    return vec2(tex_coord(coord.x, offset.u_offset), tex_coord(coord.y, offset.v_offset));
}

vec3 schlick_fresnel(float HdotV, vec3 fresnel_base) {
    return fresnel_base + (1.0 - fresnel_base) * pow(1.0 - HdotV, 5.0);
}

float ggx_normal_distribution(vec3 N, vec3 H, float a) {
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;

    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return (a2 + 0.0000001) / denom;
}

float ggx_geometry(float NdotV, float NdotL, float r2) {
    float a1 = r2 + 1.0;
    float k = a1 * a1 / 8.0;
    float denom = NdotV * (1.0 - k) + k;
    float ggx1 = NdotV / denom;
    denom = NdotL * (1.0 - k) + k;
    float ggx2 = NdotL / denom;
    return ggx1 * ggx2;
}

float s_curve (float x) {
		x = x * 2.0 - 1.0;
		return -x * abs(x) * 0.5 + x + 0.5;
}

struct PointLight {
    vec3 position;
    vec3 color;
    float intensity;
};

struct DirectionalLight {
    vec3 color;
    float intensity;
    vec3 direction;
};

struct SpotLight {
    vec3 position;
    vec3 color;
    vec3 direction;
    float angle;
    float intensity;
    float range;
    float smoothness;
};

layout(std140, set = 0, binding = 1) uniform Environment {
    vec3 ambient_color;
    vec3 camera_position;
    int point_light_count;
    int directional_light_count;
    int spot_light_count;
};

layout(std140, set = 0, binding = 2) uniform PointLights {
    PointLight plight[128];
};

layout(std140, set = 0, binding = 3) uniform DirectionalLights {
    DirectionalLight dlight[16];
};

layout(std140, set = 0, binding = 4) uniform SpotLights {
    SpotLight slight[128];
};

layout(std140, set = 1, binding = 0) uniform Material {
    UvOffset uv_offset;
    float alpha_cutoff;
};

layout(set = 1, binding = 1) uniform texture2D MyMaterial_albedo_texture;
layout(set = 1, binding = 2) uniform texture2D emission_samp;
layout(set = 1, binding = 3) uniform texture2D normal_samp;
layout(set = 1, binding = 4) uniform texture2D metallic_roughness_samp;
layout(set = 1, binding = 5) uniform texture2D ambient_occlusion_samp;
layout(set = 1, binding = 6) uniform texture2D cavity_samp;

layout(set = 3, binding = 2) uniform sampler MyMaterial_albedo_texture_sampler;

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec4 in_color;
layout(location = 3) in vec4 in_material_weights;


layout(location = 0) out vec4 out_color;

vec3 fresnel(float HdotV, vec3 fresnel_base) {
  return fresnel_base + (1.0 - fresnel_base) * pow(1.0 - HdotV, 5.0);
}

vec3 compute_light(vec3 attenuation,
                 vec3 light_color,
                 vec3 view_direction,
                 vec3 light_direction,
                 vec3 albedo,
                 vec3 normal,
                 float roughness2,
                 float metallic,
                 vec3 fresnel_base) {

  vec3 halfway = normalize(view_direction + light_direction);
  float normal_distribution = ggx_normal_distribution(normal, halfway, roughness2);

  float NdotV = max(dot(normal, view_direction), 0.0);
  float NdotL = max(dot(normal, light_direction), 0.0);
  float HdotV = max(dot(halfway, view_direction), 0.0);
  float geometry = ggx_geometry(NdotV, NdotL, roughness2);


  vec3 fresnel = fresnel(HdotV, fresnel_base);
  vec3 diffuse = vec3(1.0) - fresnel;
  diffuse *= 1.0 - metallic;

  vec3 nominator = normal_distribution * geometry * fresnel;
  float denominator = 4 * NdotV * NdotL + 0.0001;
  vec3 specular = nominator / denominator;

  vec3 resulting_light = (diffuse * albedo / PI + specular) * light_color * attenuation * NdotL;
  return resulting_light;
}

vec4 triplanar_texture(texture2D samp, float layer, vec3 blend, vec2 uv_x, vec2 uv_y, vec2 uv_z) {
  vec4 x = texture(sampler2D(samp, MyMaterial_albedo_texture_sampler), uv_x);
  vec4 y = texture(sampler2D(samp, MyMaterial_albedo_texture_sampler), uv_y);
  vec4 z = texture(sampler2D(samp, MyMaterial_albedo_texture_sampler), uv_z);
  return blend.x * x + blend.y * y + blend.z * z;
}

vec3 triplanar_normal_to_world(texture2D samp, float layer, vec3 blend, vec2 uv_x, vec2 uv_y, vec2 uv_z, vec3 surf_normal) {
  // Important that the texture is loaded as Unorm.
  vec3 tnormalx = 2.0 * texture(sampler2D(samp, MyMaterial_albedo_texture_sampler), uv_x).rgb - 1.0;
  vec3 tnormaly = 2.0 * texture(sampler2D(samp, MyMaterial_albedo_texture_sampler), uv_y).rgb - 1.0;
  vec3 tnormalz = 2.0 * texture(sampler2D(samp, MyMaterial_albedo_texture_sampler), uv_z).rgb - 1.0;

  // Use swizzle method to convert normal into world space.
  // Get the sign (-1 or 1) of the surface normal
  vec3 axis_sign = sign(surf_normal);
  // Flip tangent normal z to account for surface normal facing
  tnormalx.z *= axis_sign.x;
  tnormaly.z *= axis_sign.y;
  tnormalz.z *= axis_sign.z;
  // Swizzle tangent normals to match world orientation and triblend
  return normalize(
      tnormalx.zyx * blend.x +
      tnormaly.xzy * blend.y +
      tnormalz.xyz * blend.z
  );
}

vec4 triplanar_texture_splatted(texture2D samp, vec4 mtl_weights, vec3 blend, vec2 uv_x, vec2 uv_y, vec2 uv_z) {
  vec4 v0 = triplanar_texture(samp, 0.0, blend, uv_x, uv_y, uv_z);
  vec4 v1 = triplanar_texture(samp, 1.0, blend, uv_x, uv_y, uv_z);
  vec4 v2 = triplanar_texture(samp, 2.0, blend, uv_x, uv_y, uv_z);
  vec4 v3 = triplanar_texture(samp, 3.0, blend, uv_x, uv_y, uv_z);
  // TODO: depth maps
  return mtl_weights.r * v0 +
         mtl_weights.g * v1 +
         mtl_weights.b * v2 +
         mtl_weights.a * v3;
}

vec3 triplanar_normal_to_world_splatted(texture2D samp, vec4 mtl_weights, vec3 blend, vec2 uv_x, vec2 uv_y, vec2 uv_z, vec3 surf_normal) {
  vec3 v0 = triplanar_normal_to_world(samp, 0.0, blend, uv_x, uv_y, uv_z, surf_normal);
  vec3 v1 = triplanar_normal_to_world(samp, 1.0, blend, uv_x, uv_y, uv_z, surf_normal);
  vec3 v2 = triplanar_normal_to_world(samp, 2.0, blend, uv_x, uv_y, uv_z, surf_normal);
  vec3 v3 = triplanar_normal_to_world(samp, 3.0, blend, uv_x, uv_y, uv_z, surf_normal);
  // TODO: depth maps
  return normalize(
      mtl_weights.r * v0 +
      mtl_weights.g * v1 +
      mtl_weights.b * v2 +
      mtl_weights.a * v3
  );
}

void main() {
  // Do triplanar mapping (world space -> UVs).
  float texture_scale = 10.0;
  vec3 blend = pow(abs(in_normal), vec3(3));
  blend = blend / (blend.x + blend.y + blend.z);
  vec2 uv_x = tex_coords(in_position.zy / texture_scale, uv_offset);
  vec2 uv_y = tex_coords(in_position.xz / texture_scale, uv_offset);
  vec2 uv_z = tex_coords(in_position.xy / texture_scale, uv_offset);

  vec4 albedo_alpha       = triplanar_texture_splatted(MyMaterial_albedo_texture, in_material_weights, blend, uv_x, uv_y, uv_z);
  float alpha             = albedo_alpha.a;
  if(alpha < alpha_cutoff) discard;

  vec3 albedo             = albedo_alpha.rgb;
  vec3 emission           = triplanar_texture_splatted(emission_samp, in_material_weights, blend, uv_x, uv_y, uv_z).rgb;
  vec3 normal             = triplanar_normal_to_world_splatted(normal_samp, in_material_weights, blend, uv_x, uv_y, uv_z, in_normal);
  vec2 metallic_roughness = triplanar_texture_splatted(metallic_roughness_samp, in_material_weights, blend, uv_x, uv_y, uv_z).bg;
  float ambient_occlusion = triplanar_texture_splatted(ambient_occlusion_samp, in_material_weights, blend, uv_x, uv_y, uv_z).r;
  // TODO: Use cavity
  // float cavity            = texture(cavity, tex_coords(vertex.tex_coord, final_tex_coords).r;
  float metallic          = metallic_roughness.r;
  float roughness         = metallic_roughness.g;

  float roughness2 = roughness * roughness;
  vec3 fresnel_base = mix(vec3(0.04), albedo, metallic);

  vec3 view_direction = normalize(camera_position - in_position);
  vec3 lighted = vec3(0.0);
  for (int i = 0; i < point_light_count; i++) {
      vec3 light_direction = normalize(plight[i].position - in_position);
      float attenuation = plight[i].intensity / dot(light_direction, light_direction);

      vec3 light = compute_light(vec3(attenuation),
                                 plight[i].color,
                                 view_direction,
                                 light_direction,
                                 albedo,
                                 normal,
                                 roughness2,
                                 metallic,
                                 fresnel_base);

      lighted += light;
  }

  for (int i = 0; i < directional_light_count; i++) {
      vec3 light_direction = -normalize(dlight[i].direction);
      float attenuation = dlight[i].intensity;

      vec3 light = compute_light(vec3(attenuation),
                                 dlight[i].color,
                                 view_direction,
                                 light_direction,
                                 albedo,
                                 normal,
                                 roughness2,
                                 metallic,
                                 fresnel_base);

      lighted += light;
  }

  for (int i = 0; i < spot_light_count; i++) {
      vec3 light_vec = slight[i].position - in_position;
      vec3 normalized_light_vec = normalize(light_vec);

      // The distance between the current fragment and the "core" of the light
      float light_length = length(light_vec);

      // The allowed "length", everything after this won't be lit.
      // Later on we are dividing by this range, so it can't be 0
      float range = max(slight[i].range, 0.00001);

      // get normalized range, so everything 0..1 could be lit, everything else can't.
      float normalized_range = light_length / max(0.00001, range);

      // The attenuation for the "range". If we would only consider this, we'd have a
      // point light instead, so we need to also check for the spot angle and direction.
      float range_attenuation = max(0.0, 1.0 - normalized_range);

      // this is actually the cosine of the angle, so it can be compared with the
      // "dotted" frag_angle below a lot cheaper.
      float spot_angle = max(slight[i].angle, 0.00001);
      vec3 spot_direction = normalize(slight[i].direction);
      float smoothness = 1.0 - slight[i].smoothness;

      // Here we check if the current fragment is within the "ring" of the spotlight.
      float frag_angle = dot(spot_direction, -normalized_light_vec);

      // so that the ring_attenuation won't be > 1
      frag_angle = max(frag_angle, spot_angle);

      // How much is this outside of the ring? (let's call it "rim")
      // Also smooth this out.
      float rim_attenuation = pow(max((1.0 - frag_angle) / (1.0 - spot_angle), 0.00001), smoothness);

      // How much is this inside the "ring"?
      float ring_attenuation = 1.0 - rim_attenuation;

      // combine the attenuations and intensity
      float attenuation = range_attenuation * ring_attenuation * slight[i].intensity;

      vec3 light = compute_light(vec3(attenuation),
                                 slight[i].color,
                                 view_direction,
                                 normalize(light_vec),
                                 albedo,
                                 normal,
                                 roughness2,
                                 metallic,
                                 fresnel_base);
      lighted += light;
  }

  vec3 ambient = ambient_color * albedo * ambient_occlusion;
  vec3 color = ambient + lighted + emission;

  out_color = vec4(color, alpha) * in_color;
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
      transform: Transform::from_translation(Vec3::new(3.0, 20.0, -20.0))
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

  let texture_handle = asset_server.load("albedo.png");
  // let texture_handle = asset_server.load("elon.png");

  let mutated_chunk_keys = &terrain_resource.mutated_chunk_keys;
  let map = &terrain_resource.map;
  // let map_chunk_graphics = &terrain_resource.map_chunk_graphics;
  let chunk_shape = terrain_resource.chunk_shape.clone();

  // For each mutated chunk, and any adjacent chunk, the mesh will need to be updated. (This is
  // not 100% true, but it's a conservative assumption to make. In reality, if a given voxel is
  // mutated and it's L1 distance from an adjacent chunk is less than or equal to 2, that adjacent
  // chunk's mesh is dependent on that voxel, so it must be re-meshed).
  let mut chunk_keys_to_update: HashSet<Point3i> = HashSet::new();
  let offsets = Point3i::moore_offsets();
  for chunk_key in mutated_chunk_keys.iter() {
    chunk_keys_to_update.insert(*chunk_key);
    for offset in offsets.iter() {
      chunk_keys_to_update.insert(*chunk_key + *offset * chunk_shape);
    }
  }

  // Now we generate mesh vertices for each chunk.
  let local_cache = LocalChunkCache::new();
  let reader = ChunkMapReader3::new(map, &local_cache);
  for chunk_key in chunk_keys_to_update.into_iter() {
    let padded_chunk_extent = map.extent_for_chunk_at_key(&chunk_key).padded(1);
    let mut padded_chunk = Array3::fill(padded_chunk_extent, 0.0);
    copy_extent(&padded_chunk_extent, &reader, &mut padded_chunk);
    let mut sn_buffer = SurfaceNetsBuffer::new(padded_chunk_extent.num_points());
    surface_nets(&padded_chunk, &padded_chunk_extent, &mut sn_buffer);

    let mut indices_u32: Vec<u32> = vec![];
    for i in sn_buffer.indices {
      indices_u32.push(i as u32);
    }
    let indices_points = convert_flatu32_to_points(indices_u32.clone());

    if indices_u32.len() == 0 {
      continue;
    }

    let positions = sn_buffer.positions.clone();
    let normals = sn_buffer.normals.clone();
    let mesh = create_mesh(meshes, positions, normals, indices_u32);

    // Create as a function
    let body = RigidBodyBuilder::new_static().translation(0., 0., 0.);
    let collider = ColliderBuilder::trimesh(
      convert_arrayf32_to_points(sn_buffer.positions),
      indices_points,
    );

    // Create a green material
    let green_material: Handle<MyMaterial> = materials.add(MyMaterial {
      albedo: Color::rgb(0.0, 0.8, 0.0),
      albedo_texture: Some(texture_handle.clone()),
      shaded: true,
    });

    commands
    // cube
    .spawn(MeshComponents {
      mesh: mesh,
      render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::specialized(
        pipeline_handle.clone(),
        // NOTE: in the future you wont need to manually declare dynamic bindings
        PipelineSpecialization {
          dynamic_bindings: vec![
            DynamicBinding {
              bind_group: 0,
              binding: 0,
            },
            DynamicBinding {
              bind_group: 1,
              binding: 1,
            },
            // Transform
            DynamicBinding {
              bind_group: 2,
              binding: 0,
            },
            // MyMaterial_color
            DynamicBinding {
              bind_group: 3,
              binding: 0,
            },
          ],
          ..Default::default()
        },
      )]),
      transform: Transform::from_translation(Vec3::new(-2.0, 0.0, 0.0)),
      ..Default::default()
    })
    .with(green_material.clone())
    .with(body)
    .with(collider)
    ;


    // Create as a function
    // spawn_terrain(commands, materials, materials, terrain_resource);
    // let material = materials.add(
    //   Color::rgba(0.5, 0.4, 0.3, 1.0).into());
    // let _entity_ = commands
    //   .spawn(PbrComponents {
    //     mesh,
    //     material: material,
    //     ..Default::default()
    //   })
    //   .with(body)
    //   .with(collider)
    //   .with(PickableMesh::default())
    //   // .with(HighlightablePickMesh::default())
    //   // .with(SelectablePickMesh::new())
    //   .current_entity();
    // terrain_resource.map_chunk_graphics.insert(chunk_key, entity);
  }
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

// fn spawn_terrain(
//   mut commands: Commands,
//   mut meshes: ResMut<Assets<Mesh>>,
//   mut materials: ResMut<Assets<StandardMaterial>>,

//   mut terrain_resource: ResMut<TerrainResource>,
// ) {
//   let material = materials.add(
//     Color::rgba(0.5, 0.4, 0.3, 1.0).into());
//   let _entity_ = commands
//     .spawn(PbrComponents {
//       mesh,
//       material: material,
//       ..Default::default()
//     })
//     .with(body)
//     .with(collider)
//     .with(PickableMesh::default())
//     // .with(HighlightablePickMesh::default())
//     // .with(SelectablePickMesh::new())
//     .current_entity();
// }

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

