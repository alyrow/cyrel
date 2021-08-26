#! /bin/bash

. ./dependencies.config

mkdir "./public/dependencies"
mkdir "./public/dependencies/semantic"

echo Downloading jquery...
wget -O ./public/dependencies/jquery.min.js "$JQUERY"

echo Downloading fomantic-ui...
wget -O fomantic.zip "$FOMANTIC"
unzip fomantic.zip "Fomantic-UI-master/dist/semantic.min.*" -d "./public/dependencies/semantic" -j
unzip fomantic.zip "Fomantic-UI-master/dist/*/**" -d "./public/dependencies/semantic" -j
rm fomantic.zip
find ./public/dependencies/semantic/Fomantic-UI-master/dist/ -maxdepth 1 -print -exec mv {} ./public/dependencies/semantic/ \;
rm -rf ./public/dependencies/semantic/Fomantic-UI-master/

echo Downloading json-rpc 2.0 client...
wget -O ./public/dependencies/json-rpc.min.js "$JSONRPC"