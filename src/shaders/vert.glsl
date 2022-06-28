#version 300 es
precision highp float;

uniform mat4 u_proj;
uniform mat4 u_view;
uniform mat4 u_view_inv;
uniform float u_scale;
in vec4 i_pos;
in vec4 a_pos;
in vec3 a_normal;
out vec3 v_normal;
out vec3 v_lightdir;

void main() {
    vec3 lightdir = vec3(0.0, 1.0, 1.0);
    vec4 pos = i_pos + (a_pos * u_scale);
    gl_Position = u_proj * u_view * pos;
    v_normal = a_normal;
    v_lightdir = mat3(u_view_inv) * lightdir;
}
