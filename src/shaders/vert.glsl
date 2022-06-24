#version 300 es
precision highp float;

uniform mat4 u_proj;
uniform mat4 u_view;
in mat4 i_model;
in vec4 a_pos;
in vec3 a_normal;
out vec3 v_normal;
out vec3 v_lightdir;

void main() {
    vec3 lightdir = vec3(0.0, 1.0, 1.0);
    gl_Position = u_proj * u_view * i_model * a_pos;
    v_normal = mat3(transpose(inverse(i_model))) * a_normal;
    v_lightdir = mat3(inverse(u_view)) * lightdir;
}
