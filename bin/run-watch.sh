#!/bin/bash

SCRIPT_DIR=`dirname ${0}`
cd $SCRIPT_DIR/..

cargo watch -s 'mold -run cargo run' -w "src"
