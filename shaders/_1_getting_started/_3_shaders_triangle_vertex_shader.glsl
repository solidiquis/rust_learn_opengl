#version 330 core
in vec3 aPos;
in vec3 aCol;
out vec3 vCol;

void main() {
  gl_Position = vec4(aPos, 1.0);
  vCol = aCol;
}
