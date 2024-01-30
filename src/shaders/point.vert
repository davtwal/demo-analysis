// VERTEX SHADER
#version 330 core
uniform vec4 world_bounds; // min_x, min_y, max_x, max_y

layout (location = 0) in vec4 color;
layout (location = 1) in vec3 point; // Point in world space

out vec4 vert_color;

void main() {
    vec2 diff = vec2(world_bounds.z - world_bounds.x,
                     world_bounds.w - world_bounds.y);

    //vec2 diff = vec2(1.0, 1.0);

    vert_color = color;
    gl_Position = vec4(point.x / diff.x,
                       point.y / diff.y,
                       0.0, 1.0);
}