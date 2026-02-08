#version 100

// a modified version of: https://github.com/kiwipxl/GLSL-shaders/blob/master/bloom.glsl

precision highp float;

uniform lowp float scale;
uniform sampler2D _ScreenTexture;
varying vec2 uv;

void main() {
    float uv_x = uv.x;
    float uv_y = 1.0-uv.y;
    float bloom_spread = 4.0 / (256.0 * scale);
    float bloom_intensity = 2.0;

    vec4 sum = vec4(0.0);
    for (int n = 0; n < 9; ++n) {
        uv_y = 1.0-uv.y + (bloom_spread * float(n - 4));
        vec4 h_sum = vec4(0.0);
        h_sum += texture2D(_ScreenTexture, vec2(uv_x - (4.0 * bloom_spread), uv_y));
        h_sum += texture2D(_ScreenTexture, vec2(uv_x - (3.0 * bloom_spread), uv_y));
        h_sum += texture2D(_ScreenTexture, vec2(uv_x - (2.0 * bloom_spread), uv_y));
        h_sum += texture2D(_ScreenTexture, vec2(uv_x - bloom_spread, uv_y));
        h_sum += texture2D(_ScreenTexture, vec2(uv_x, uv_y));
        h_sum += texture2D(_ScreenTexture, vec2(uv_x + bloom_spread, uv_y));
        h_sum += texture2D(_ScreenTexture, vec2(uv_x + (2.0 * bloom_spread), uv_y));
        h_sum += texture2D(_ScreenTexture, vec2(uv_x + (3.0 * bloom_spread), uv_y));
        h_sum += texture2D(_ScreenTexture, vec2(uv_x + (4.0 * bloom_spread), uv_y));
        sum += h_sum / 9.0;
    }
    
    sum = sum / 9.0 * bloom_intensity;
    sum = sum * sum;
    gl_FragColor = vec4(sum.x,sum.y,sum.z,0.1);
}