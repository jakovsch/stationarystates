#version 300 es
precision highp float;

const float hbar_me = 1.15767636e-4;
uniform float u_dt;
in vec3 i_pos;
flat out vec3 v_pos;

vec2 conj(in vec2 z) {
    return vec2(z.x, -z.y);
}

float conj_mul(in vec2 z) {
    return dot(z, z);
}

void main() {
    float v = 0.0;
    v_pos = i_pos+v*u_dt;
}
