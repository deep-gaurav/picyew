set -x
curl https://sh.rustup.rs -sSf | sh -s - --default-toolchain stable -y
source ~/.cargo/env

curl -L https://github.com/rustwasm/wasm-pack/releases/download/v0.9.1/wasm-pack-v0.9.1-x86_64-unknown-linux-musl.tar.gz --output wasm-pack-v0.9.1-x86_64-unknown-linux-musl.tar.gz

ls -l
tar -zxvf wasm-pack-v0.9.1-x86_64-unknown-linux-musl.tar.gz

export PATH="$PATH:$PWD/wasm-pack-v0.9.1-x86_64-unknown-linux-musl"

npm install -g rollup

wasm-pack build --target web --out-name package
rollup ./main.js --format iife --file ./pkg/bundle.js

mkdir -p dist
cp -r static/* dist/
cp pkg/bundle.js dist/
cp pkg/package_bg.wasm dist/