#version 120
precision lowp float;

varying vec2 uv;
uniform sampler2D Texture;

uniform lowp float y;
uniform lowp float height;
uniform lowp float maxTowerHeight;

void main() {
    float value = ((1.0-uv.y) * height - y);
    
    vec3 colors[3] = vec3[3](
        vec3(0.110,0.718,1.000),
        vec3(0.020,0.318,0.647),
        vec3(0.118,0.157,0.318)
    );
    
    if (value >= maxTowerHeight) {
        gl_FragColor = vec4(colors[colors.length()-1],1.0);
    } else if (value <= 0.0) {
        gl_FragColor = vec4(colors[0],1.0);
    }
    else {
        float maxValue = maxTowerHeight / float(colors.length()-1);

        float stepLow = floor(value / maxValue);
        float stepHigh = ceil(value / maxValue);

        float diff = (stepHigh - value / maxValue);

        vec3 col = mix(colors[int(stepLow)],colors[int(stepHigh)],1.0-diff);

        gl_FragColor = vec4(col,1.0);
    }
}