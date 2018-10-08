#!/bin/sh

docker run --rm  \
 -v ${PWD}:/home/rust/src \
 -v cargo-registry:/home/rust/.cargo/registry \
 -p 8000:8000 \
 ekidd/rust-musl-builder:nightly \
 $@
