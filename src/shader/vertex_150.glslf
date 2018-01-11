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

const vec3 DIRECTION = normalize(vec3(0, 1, 0.5));

void main() {
    Target0 = texture(t_Image, v_Uv) * (0.6 + 0.4*abs(dot(v_Normal, DIRECTION)));
}
