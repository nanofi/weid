#!/bin/bash

docker run -it --rm  \
 -v ${PWD}:/home/rust/src \
 ekidd/rust-musl-builder:nightly \
 $@
