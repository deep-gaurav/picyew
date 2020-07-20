rm -rf dist
mkdir -p dist
cp ./static/* dist/
cp ./pkg/bundle.js dist/
cp ./pkg/package_bg.wasm dist/