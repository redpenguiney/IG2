#version 460 // TODO: DON'T USE THIS SHADER VERSION

layout(location=0) in vec3 vertexPos;
layout(location=1) in vec4 vertexColor;
layout(location=2) in vec3 vertexNormal;
layout(location=3) in vec2 texCoords;
layout(location=4) in mat4 model;
// locations 5-7 are part of model
layout(location=8) in float textureZ;

uniform mat4 proj;
uniform mat4 camera;
uniform mat4 modelToLightSpace;
//layout (std430, binding=3) buffer model_array
//{
  //  mat4 model[10000];
//};

out vec3 fragmentColor;
out vec3 fragmentNormal;
out vec3 fragmentTexCoords;
out vec4 lightSpaceCoords;

void main()
{
    // todo: an easy optimization would be multiplying proj by camera on the cpu once instead of for every vertex
    gl_Position = proj * camera * model * vec4(vertexPos, 1.0); // TODO: draw_id issue (check shadow shaders too!)
    fragmentColor = vertexColor.xyz;
    fragmentNormal = vertexNormal;
    fragmentTexCoords = vec3(texCoords.xy, textureZ);
    //lightSpaceCoords = modelToLightSpace * model * vec4(vertexPos, 1.0);
}          