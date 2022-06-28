#version 300 es
precision highp float;

uniform mat4 u_proj;
uniform mat4 u_view;
uniform float u_scale;
in vec4 i_pos;
in vec4 a_pos;
out vec3 v_pos;
out float v_radius;

void main() {
    vec4 pos = i_pos + (a_pos * u_scale * 5.0);
    gl_Position = u_proj * u_view * pos;
    v_radius = u_scale;
    v_pos = vec3(u_view * i_pos);
}
