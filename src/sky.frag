#version 100
precision lowp float;

varying vec2 uv;
uniform sampler2D Texture;

uniform lowp float y;
uniform lowp float height;
uniform lowp float maxTowerHeight;

void main() {
    float value = ((1.0-uv.y) * height - y);
    
    vec3 colorA = vec3(0.110,0.718,1.000);
    vec3 colorB = vec3(0.020,0.318,0.647);
    vec3 colorC = vec3(0.118,0.157,0.318);

    
    if (value >= maxTowerHeight) {
        gl_FragColor = vec4(colorC,1.0);
    } else if (value <= 0.0) {
        gl_FragColor = vec4(colorA,1.0);
    }
    else {
        float maxValue = maxTowerHeight / float(2);

        float stepLow = floor(value / maxValue);
        float stepHigh = ceil(value / maxValue);

        float diff = (stepHigh - value / maxValue);

        vec3 colorLow = colorA;
        if (int(stepLow) == 1) {
            colorLow = colorB;
        } else if (int(stepLow) == 2){
            colorLow = colorC;
        }
        vec3 colorHigh = colorA;
        if (int(stepHigh) == 1) {
            colorHigh = colorB;
        } else if (int(stepHigh) == 2){
            colorHigh = colorC;
        }
        vec3 col = mix(colorLow,colorHigh,1.0-diff);

        gl_FragColor = vec4(col,1.0);
    }
}