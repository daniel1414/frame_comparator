#!/bin/bash


echo "compiling vertex shader..."
if ! glslc shader.vert -o vert.spv; then
    echo "Error: Failed to compile vertex shader!"
    exit 1
fi
echo "success!"

echo "compiling fragment shader..."
if ! glslc shader.frag -o frag.spv; then 
    echo "Error: Failed to compile fragment shader!"
    exit 2
fi
echo "success!"
