#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_functions

#import bevy_pbr::pbr_types
#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::fog
// #import bevy_pbr::pbr_functions
#import bevy_pbr::pbr_ambient

#ifdef TONEMAP_IN_SHADER
#import bevy_core_pipeline::tonemapping
#endif


@group(1) @binding(0)
var albedo: texture_2d_array<f32>;
@group(1) @binding(1)
var albedo_sampler: sampler;
@group(1) @binding(2)
var normal: texture_2d_array<f32>;
@group(1) @binding(3)
var normal_sampler: sampler;


struct Vertex {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
};

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) world_position: vec4<f32>,
  @location(1) world_normal: vec3<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
  var out: VertexOutput;
  out.world_position = mesh_position_local_to_world(mesh.model, vec4<f32>(vertex.position, 1.0));
  out.clip_position = mesh_position_local_to_clip(mesh.model, vec4<f32>(vertex.position, 1.0));
  out.world_normal = vertex.normal;
  return out;
}

struct FragmentInput {
  // @builtin(position) frag_coord: vec4<f32>,
  @builtin(front_facing) is_front: bool,
  @builtin(position) frag_coord: vec4<f32>,

  @location(0) world_position: vec4<f32>,
  @location(1) world_normal: vec3<f32>,

  // #import bevy_pbr::mesh_vertex_output
};

@fragment
fn fragment(input: FragmentInput) -> @location(0) vec4<f32> {
  var zy = input.world_position.zy % 1.0;
  if zy.x < 0.0 {
    zy.x += 1.0;
  }
  if zy.y < 0.0 {
    zy.y += 1.0;
  }

  var xz = input.world_position.xz % 1.0;
  if xz.x < 0.0 {
    xz.x += 1.0;
  }
  if xz.y < 0.0 {
    xz.y += 1.0;
  }

  var xy = input.world_position.xy % 1.0;
  if xy.x < 0.0 {
    xy.x += 1.0;
  }
  if xy.y < 0.0 {
    xy.y += 1.0;
  }

  var dx = textureSample(albedo, albedo_sampler, zy, i32(1));
  var dy = textureSample(albedo, albedo_sampler, xz, i32(1));
  var dz = textureSample(albedo, albedo_sampler, xy, i32(1));

  let dx_normal = dpdx(input.world_position);
  let dy_normal = dpdy(input.world_position);
  // let cross = cross(dx_normal, dy_normal); // Error in WebGPU
  // let normal = normalize(cross(dx_normal, dy_normal));
  let normal = input.world_normal;


  let sharpness = 10.0;
  var weights = pow(abs(normal.xyz), vec3<f32>(sharpness, sharpness, sharpness));
  weights = weights / (weights.x + weights.y + weights.z);

  var color = dx * weights.x + dy * weights.y + dz * weights.z;
  return color;


  // var pbr_input: PbrInput = pbr_input_new();
  // pbr_input.material.base_color = pbr_input.material.base_color * color;
  // pbr_input.frag_coord = input.frag_coord;
  // pbr_input.world_position = input.world_position;
  // pbr_input.world_normal = prepare_world_normal(
  //   input.world_normal,
  //   true,
  //   false,
  // );

  // pbr_input.is_orthographic = view.projection[3].w == 1.0;

  // let sharpness_1 = 8.0;
  // var weights_1 = pow(abs(input.world_normal), vec3(sharpness_1));
  // weights_1 = weights_1 / (weights_1.x + weights_1.y + weights_1.z);

  // let scale = 1.0;
  // let uv_x = input.world_position.yz * scale;
  // let uv_y = input.world_position.zx * scale;
  // let uv_z = input.world_position.xy * scale;
  // var triplanar = Triplanar(weights_1, uv_x, uv_y, uv_z);

  // pbr_input.N = triplanar_normal_to_world_splatted(
  //   input.voxel_weight, 
  //   input.world_normal, 
  //   input.voxel_type_1, 
  //   triplanar
  // );

  // pbr_input.V = calculate_view(input.world_position, pbr_input.is_orthographic);

  // return tone_mapping(pbr(pbr_input));

  // return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}




