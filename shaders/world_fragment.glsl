#version 330

in vec3 fragmentColor;
in vec3 fragmentNormal;
in vec3 fragmentTexCoords;
in vec4 lightSpaceCoords;

layout(location = 0) out vec4 Output;

uniform sampler2DArray textures;
uniform sampler2D shadowmap;

float getShadow(vec4 pos) {
    vec3 projCoords = pos.xyz / pos.w;
    projCoords = projCoords * 0.5 + 0.5;
    if (projCoords.z > 1.0) {return 1.0;}
    float closestDepth = texture(shadowmap, projCoords.xy).r;   
    float currentDepth = projCoords.z;
    float bias = 0.001;  
    float shadow = currentDepth - bias > closestDepth  ? 0.4 : 1.0;  
    return shadow;
}

void main()
{
    vec4 tx;
    if (fragmentTexCoords.z == -1.0) {
        tx = vec4(1.0, 1.0, 1.0, 1.0);
    }
    else {
        tx = texture(textures, fragmentTexCoords);
    }
    if (tx.a < 0.1) {
        discard;
    };
    vec4 color = vec4(fragmentColor, 1) * tx;
    float brightness = 1.0; //getShadow(lightSpaceCoords);
    Output = vec4(color.xyz * vec3(brightness, brightness, brightness), 0);
    //Output = texture(shadowmap, fragmentTexCoords.xy);
    //Output = vec4(lightSpaceCoords.xyz/lightSpaceCoords.w, 0.0);
}