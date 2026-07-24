#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

echo "Building waffle.wasm..."
cargo build --release --target wasm32-unknown-unknown

echo "Generating JS bindings..."
mkdir -p web/pkg
wasm-bindgen --out-dir web/pkg --target web --no-typescript \
  target/wasm32-unknown-unknown/release/waffle.wasm

cp style.css web/style.css
cat > web/index.html <<'EOF'
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Waffle — Spelling Bee</title>
    <link rel="stylesheet" href="style.css" />
    <link rel="preconnect" href="https://fonts.googleapis.com" />
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
    <link
      href="https://fonts.googleapis.com/css2?family=Fraunces:opsz,wght@9..144,400;9..144,600;9..144,700&family=IBM+Plex+Mono:wght@400;500&display=swap"
      rel="stylesheet"
    />
  </head>
  <body>
    <script type="module">
      import init from "./pkg/waffle.js";
      init("./pkg/waffle_bg.wasm");
    </script>
  </body>
</html>
EOF

echo "Done → web/"
