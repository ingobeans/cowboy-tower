#version 100

// stolen from https://github.com/Experience-Monks/glsl-fast-gaussian-blur
// :3

precision mediump float;

uniform sampler2D _ScreenTexture;
uniform vec2 res;
varying vec2 uv;

vec4 blur9(sampler2D image, vec2 uv, vec2 resolution, vec2 direction) {
  vec4 color = vec4(0.0);
  vec2 off1 = vec2(1.3846153846) * direction;
  vec2 off2 = vec2(3.2307692308) * direction;
  color += texture2D(image, uv) * 0.2270270270;
  color += texture2D(image, uv + (off1 / resolution)) * 0.3162162162;
  color += texture2D(image, uv - (off1 / resolution)) * 0.3162162162;
  color += texture2D(image, uv + (off2 / resolution)) * 0.0702702703;
  color += texture2D(image, uv - (off2 / resolution)) * 0.0702702703;
  return color;
}
void main() {
	vec2 flippedUv = vec2(uv.x, 1.0-uv.y);
	gl_FragColor = blur9(_ScreenTexture,flippedUv,res,vec2(1.0,1.0));
}