#version 300 es
precision mediump float;

uniform sampler2D s_color;
uniform sampler2D s_occlusion;

layout (location = 0) out vec4 o_color;

void main() {
    ivec2 xy = ivec2(gl_FragCoord.xy);
    vec4 color = texelFetch(s_color, xy, 0);
    float occlusion = texelFetch(s_occlusion, xy, 0).r;
    o_color = vec4(
        clamp(color.rgb - occlusion, 0.0, 1.0),
        color.a
    );
}
