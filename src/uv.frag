in vec4 vert_colour;
in vec2 uv;

out vec4 frag_colour;

uniform sampler2D atlas;

void main() {
    frag_colour = texture(atlas, uv) * vert_colour;
}


