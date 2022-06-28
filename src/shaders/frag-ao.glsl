#version 300 es
precision mediump float;

const float pi = 3.141592656;
uniform sampler2D u_render_texture;
uniform vec2 u_resolution;
uniform float u_fov;
in vec3 v_pos;
in float v_radius;
out vec4 o_color;

float sphere_ao(in vec3 sph, in vec3 pos, in vec3 norm) {
    vec3 dir = sph - pos;
    float l = length(dir);
    float nl = dot(norm, dir / l);
    float h = l / v_radius;
    float h2 = h * h;
    float k2 = 1.0 - h2 * nl * nl;
    float res = max(0.0, nl) / h2;

    if (k2 > 0.0 && l > v_radius) {
        res = nl * acos(-nl * sqrt((h2 - 1.0) / (1.0 - nl * nl))) - sqrt(k2 * (h2 - 1.0));
        res = res / h2 + atan(sqrt(k2 / (h2 - 1.0)));
        res /= pi;
    }

    return res;
}

void main() {
    vec2 xy = gl_FragCoord.xy / u_resolution;
    vec4 frag = texture2D(u_render_texture, xy);
    float tg_h_fov = tan(u_fov / 2.0);
    float z = frag.a;
    vec3 norm = vec3(
        frag.x, frag.y,
        sqrt(1.0 - frag.x * frag.x - frag.y * frag.y),
    );
    vec3 view_ray = vec3(
        (xy.x * 2.0 - 1.0) * tg_h_fov * u_resolution.x / u_resolution.y,
        (xy.y * 2.0 - 1.0) * tg_h_fov, -1.0,
    );
    vec3 pos = view_ray * -z;
    float res = sphere_ao(v_pos, pos, norm);
    o_color = vec4(res, 0.0, 0.0, 1.0);
}
