#version 460
layout (location = 0) in vec3 aPos;
layout (location = 4) in mat4 model;

uniform mat4 projection;
uniform mat4 camera;
//layout (std430, binding=3) buffer model_array
//{
//    mat4 model[16];
//};


void main()
{
    gl_Position = projection * camera * model * vec4(aPos, 1.0);
}  