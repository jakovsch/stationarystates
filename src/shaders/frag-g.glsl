#version 300 es
precision mediump float;

uniform vec3 u_lightdir;
smooth in vec3 v_normal;
smooth in vec4 v_pos;
layout (location = 0) out vec4 o_color;
layout (location = 1) out vec4 o_gdata;

void main() {
    vec3 normal = normalize(v_normal);
    float ambient = 0.5;
    float diffuse = dot(normal, u_lightdir);
    float light = ambient + max(diffuse, 0.0);
    o_color = vec4(0.2, 1.0, 0.2, 1.0);
    o_color.rgb *= light;
    o_gdata = vec4(v_normal, v_pos.z);
}
