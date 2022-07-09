#version 300 es
precision highp float;

in vec4 a_pos;

void main() {
    gl_Position = a_pos;
}
