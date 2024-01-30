// PIXEL SHADER
#version 330 core

precision mediump float;
in vec4 vert_color;
out vec4 pix_color;

void main() {
    pix_color = vert_color;
}