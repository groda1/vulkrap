#!/bin/sh

echo "Building shaders"

glslc src/shaders/simple_triangle.vert -o resources/shaders/simple_triangle_vert.spv
glslc src/shaders/simple_triangle.frag -o resources/shaders/simple_triangle_frag.spv
