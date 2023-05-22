#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_functions

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



#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::pbr_ambient
#import bevy_pbr::shadows
#import bevy_pbr::fog
#import bevy_pbr::pbr_types
#import bevy_pbr::pbr_functions



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
    var pbr_input: PbrInput = pbr_input_new();
    return tone_mapping(pbr(pbr_input));

    // return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}




