#version 150 core

in vec2 v_Uv;
in vec3 v_Normal;

uniform PlayerData {
    vec3 u_Direction;
};

uniform sampler2D t_Image;

out vec4 Target0;

/*float max(float f1, float f2) {
    return f1 > f2 ? f1 : f2;
}*/

void main() {
    Target0 = texture(t_Image, v_Uv) * (0.8 + 0.2*max(-dot(v_Normal, u_Direction), 0));
}
