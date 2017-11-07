#version 150 core

in vec4 a_Pos;
in vec2 a_Uv;

uniform Transform {
    mat4 u_ViewProj;
    mat4 u_Model;
};

out vec4 v_Color;
out vec2 v_Uv;

void main() {
    gl_Position = u_ViewProj * u_Model * a_Pos;
    v_Uv = a_Uv;
}