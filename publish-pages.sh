#!/bin/bash

set -eu

cd "$(dirname "$0")"

yarn build

revision=$(git rev-parse HEAD)
working_dir=$(mktemp -d /tmp/jfrv.XXXXXX)

git clone git@github.com:ocadaruma/jfrv.git --branch gh-pages $working_dir/jfrv

rm -rf $working_dir/jfrv/*
cp -r dist/* $working_dir/jfrv/

git -C $working_dir/jfrv add .
git -C $working_dir/jfrv commit -m "pages $revision"
git -C $working_dir/jfrv push origin gh-pages

rm -rf $working_dir
