#version 150 core

in vec4 a_Pos;
in vec2 a_Uv;
in vec3 a_Normal;

uniform Transform {
    mat4 u_ViewProj;
    mat4 u_Model;
};

out vec2 v_Uv;
out vec3 v_Normal;

void main() {
    gl_Position = u_ViewProj * u_Model * a_Pos;
    v_Uv = a_Uv;
    v_Normal = a_Normal;
}
