#version 300 es
precision highp float;

uniform mat4 u_proj;
uniform mat4 u_view;
uniform float u_scale;
in vec4 i_pos;
in vec4 a_pos;
in vec3 a_normal;
smooth out vec3 v_normal;
smooth out vec4 v_pos;

void main() {
    vec4 pos = (a_pos * u_scale) + i_pos;
    v_normal = mat3(u_view) * a_normal;
    v_pos = u_view * pos;
    gl_Position = u_proj * v_pos;
}
