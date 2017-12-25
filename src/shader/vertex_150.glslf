#version 150 core

in vec2 v_Uv;

uniform sampler2D t_Image;

out vec4 Target0;

void main() {
    Target0 = texture(t_Image, v_Uv);
}
