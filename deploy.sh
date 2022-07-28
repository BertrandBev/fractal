#!/usr/bin/env sh

# abort on errors
set -e

# build
wasm-pack build --target web --release

# navigate into the build output directory
cd pkg
rm .gitignore

# Copy web files
cp ../src/web/index.html .
cp ../src/web/index.js .

# Commit repo
git init
git add -A
git commit -m 'deploy'
git push -f git@github.com:BertrandBev/fractal.git master:gh-pages

# Nav back
cd -