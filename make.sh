#!/bin/bash

set -e # Exit on error

# Detect OS
OS="$(uname -s)"
case "$OS" in
Darwin) echo "🖥️ Detected macOS" ;;
Linux) echo "🐧 Detected Linux" ;;
MINGW* | MSYS* | CYGWIN*) echo "🖥️ Detected Windows" ;;
*)
  echo "❌ Unsupported OS: $OS"
  exit 1
  ;;
esac

# Get project name (assumes the script is in the project root)
PROJECT_NAME=$(basename "$PWD")

clean() {
  cargo clean
}

build_native() {
  echo "🚀 Building native version..."
  cargo build --release
  echo "🏃 Running native executable..."
  ./target/release/$PROJECT_NAME
}

run_native() {
  cargo run --release
}

build_wasm() {
  echo "🔄 Cleaning previous WASM build..."
  rm -rf pkg
  echo "🚀 Building WebAssembly (WASM) version..."
  wasm-pack build --target web --dev --out-dir pkg
  echo "✅ WASM build complete. Output in ./pkg/"
}

run_wasm() {
  build_wasm
  simple-http-server
}

build_all() {
  build_native
  build_wasm
}

# Call function based on argument
${1:-build_all}
