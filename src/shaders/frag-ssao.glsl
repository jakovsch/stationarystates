#version 300 es
precision mediump float;

const float pi = 3.141592656;
const float tg_h_fov = 0.411054915;
uniform sampler2D s_gdata;
uniform float u_width;
uniform float u_height;
layout (location = 0) out float o_occlusion;

void main() {
    vec2 xy = gl_FragCoord.xy;
    vec4 data = texelFetch(s_gdata, ivec2(xy), 0);
    float z = data.w;
    vec3 norm = vec3(
        data.x, data.y,
        sqrt(1.0 - data.x * data.x - data.y * data.y)
    );
    vec3 view_ray = vec3(
        (xy.x / u_width * 2.0 - 1.0) * tg_h_fov * u_width / u_height,
        (xy.y / u_height * 2.0 - 1.0) * tg_h_fov, -1.0
    );
    vec3 pos = view_ray * -z;
    o_occlusion = pos.x * 0.0;
}
