#!/usr/bin/bash

clang src/ctor.c -target wasm32-unknown-unknown -O3 -c -o lib/libctor-wasm32.a
clang src/ctor.c -target wasm64-unknown-unknown -O3 -c -o lib/libctor-wasm64.a
