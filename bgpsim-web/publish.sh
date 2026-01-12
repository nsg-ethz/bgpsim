#!/usr/bin/env bash

bgpsim_version=$(cargo metadata --no-deps --format-version 1 | jq -r ".packages[] | select(.name == \"bgpsim\").version" | sed "s/\./_/g")

# clone the repo (first delete if it exists.)
if [ -d "bgpsim.github.io" ]; then
   \rm -rf "bgpsim.github.io"
fi
git clone --depth 1 git@github.com:bgpsim/bgpsim.github.io.git

# build the web-app
trunk build --release || exit 1
# remove all old files
\rm bgpsim.github.io/*.js
\rm bgpsim.github.io/*.wasm
\rm bgpsim.github.io/*.svg
\rm bgpsim.github.io/*.css
\rm bgpsim.github.io/*.html
\rm bgpsim.github.io/mapping/*.cbor
# copy the new files over
cp -r dist/* "bgpsim.github.io/"

# build the website again, but this time with a different path.
trunk build --release --public-url "/v${bgpsim_version}/"  || exit 1
# remove the old directory
if [ -d "bgpsim.github.io/v$bgpsim_version" ]; then
   \rm -rf "bgpsim.github.io/v$bgpsim_version"
fi
# copy the web-app into the repo.
mkdir -p "bgpsim.github.io/v$bgpsim_version"
cp -r dist/* "bgpsim.github.io/v$bgpsim_version"

# create the commit
cd bgpsim.github.io
git add .
git commit -m "Update"
git push origin main
cd ..
\rm -rf bgpsim.github.io
