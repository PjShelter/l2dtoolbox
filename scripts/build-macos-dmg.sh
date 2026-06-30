#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

target_args=()
target_dir="src-tauri/target/release"
arch="$(uname -m)"

case "${1:-}" in
  "")
    ;;
  "--universal")
    target_args=(--target universal-apple-darwin)
    target_dir="src-tauri/target/universal-apple-darwin/release"
    arch="universal"
    ;;
  *)
    echo "Usage: $0 [--universal]" >&2
    exit 2
    ;;
esac

export APPLE_SIGNING_IDENTITY="${APPLE_SIGNING_IDENTITY:--}"

if [[ ${#target_args[@]} -eq 0 ]]; then
  pnpm tauri build --bundles app
else
  pnpm tauri build "${target_args[@]}" --bundles app
fi

product_name="$(node -e "const fs = require('fs'); const c = JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json', 'utf8')); console.log(c.productName)")"
version="$(node -e "const fs = require('fs'); const c = JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json', 'utf8')); console.log(c.version)")"

app_bundle="${target_dir}/bundle/macos/${product_name}.app"
dmg_dir="${target_dir}/bundle/dmg"
staging_dir="${target_dir}/bundle/dmg-staging"
dmg_path="${dmg_dir}/${product_name}_${version}_${arch}.dmg"
trap 'rm -rf "$staging_dir"' EXIT

if [[ ! -d "$app_bundle" ]]; then
  echo "Expected app bundle not found: $app_bundle" >&2
  exit 1
fi

rm -rf "$staging_dir"
mkdir -p "$staging_dir" "$dmg_dir"
cp -R "$app_bundle" "$staging_dir/"
ln -s /Applications "$staging_dir/Applications"

rm -f "$dmg_path"
hdiutil create -volname "$product_name" -srcfolder "$staging_dir" -ov -format UDZO "$dmg_path"
hdiutil verify "$dmg_path"

echo "Created DMG: $dmg_path"
