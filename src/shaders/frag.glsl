#version 300 es
precision mediump float;

in vec3 v_normal;
in vec3 v_lightdir;
out vec4 o_color;

void main() {
    vec3 normal = normalize(v_normal);
    vec3 lightdir = normalize(v_lightdir);
    float ambient = 0.5;
    float diffuse = dot(normal, lightdir);
    float light = ambient + diffuse;
    o_color = vec4(0.2, 1.0, 0.2, 1.0);
    o_color.rgb *= light;
}
