#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in vec2 Vertex_Uv;

layout(set = 0, binding = 0) uniform CameraViewProj {
  mat4 ViewProj;
  mat4 View;
  mat4 InverseView;
  mat4 Projection;
  vec3 WorldPosition;
  float near;
  float far;
  float width;
  float height;
};

layout(set = 2, binding = 0) uniform Mesh {
  mat4 Model;
  mat4 InverseTransposeModel;
  uint flags;
};

void main() {
  gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
}


/* #version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in vec2 Vertex_Uv;
layout(location = 3) in uint Vertex_Layer; // New thing

layout(location = 0) out vec3 v_Position;
layout(location = 1) out vec3 v_Normal;
layout(location = 2) out vec3 v_Uv;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};

layout(set = 2, binding = 0) uniform Transform {
    mat4 Model;
};

void main() {
    v_Normal = mat3(Model) * Vertex_Normal;
    v_Position = (Model * vec4(Vertex_Position, 1.0)).xyz;

    // Gets used here and passed to the fragment shader.
    v_Uv = vec3(Vertex_Uv, Vertex_Layer);

    gl_Position = ViewProj * vec4(v_Position, 1.0);
} */