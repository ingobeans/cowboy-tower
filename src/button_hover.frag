#version 100
precision lowp float;

varying vec2 uv;

uniform lowp vec4 color;

uniform sampler2D Texture;

void main() {
    gl_FragColor = texture2D(Texture, uv) * 2.0;
}