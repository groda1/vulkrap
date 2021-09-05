#!/bin/sh

echo "Compiling shaders..."

rm resources/shaders/*.spv

for file in src/shaders/*.vert
do
	base=`basename $file`
	output="resources/shaders/${base%%.*}_vert.spv"
	echo "Compiling $file to $output"
	glslc $file -o $output
done



for file in src/shaders/*.frag
do
	base=`basename $file`
	output="resources/shaders/${base%%.*}_frag.spv"
	echo "Compiling $file to $output"
	glslc $file -o $output
done

echo "Compilation done."
