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

echo "compiling vertex depth shader..."
if ! glslc depth_shader.vert -o depth_vert.spv; then
    echo "Error: Failed to compile vertex shader!"
    exit 1
fi
echo "success!"

echo "compiling fragment depth shader..."
if ! glslc depth_shader.frag -o depth_frag.spv; then 
    echo "Error: Failed to compile fragment shader!"
    exit 2
fi


echo "success!"
