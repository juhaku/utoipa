#!/bin/bash

# Publishes crate to crates.io

token=""
ref=""
while true; do
  case $1 in
    "--token")
      shift
      token="$1"
      shift
    ;;
    "--ref")
      shift
      ref=${1/refs\/tags\//}
      shift
    ;;
    *)
      break
    ;;
  esac
done

if [[ "$token" == "" ]]; then
  echo "Missing --token <token> option argument, cannot publish crates without it!" && exit 1
fi
if [[ "$ref" == "" ]]; then
  echo "Missing --ref <ref> option argument, need an explisit ref to release!" && exit 1
fi

function publish {
  module="$1"
  if [[ "$module" == "utoipa" ]]; then
    cargo publish
  else
    cargo publish -p "$module"
  fi
}

if [ ! -f "Cargo.toml" ]; then
  echo "Missing Cargo.toml file, not in a Rust project root?" && exit 1
fi

echo "$token" | cargo login
while read -r module; do
  # crate=$(echo "$ref" | sed 's|-[0-9]*\.[0-9]*\.[0-9].*||')
  crate=${ref/-[0-9]\.[0-9]\.[0-9]*/}
  if [[ "$crate" != "$module" ]]; then
    echo "Module: $module does not match to release crate: $crate, skipping release for module"
    continue
  fi
  
  toml="Cargo.toml"
  if [[ "$module" != "utoipa" ]]; then
    toml="$module/Cargo.toml"
  fi
  
  current_version=$(cargo read-manifest --manifest-path "$toml" | jq -r '.version')
  last_version=$(curl -sS https://crates.io/api/v1/crates/"$module"/versions | jq -r '.versions[0].num')
  if [[ "$last_version" == "$current_version" ]]; then
    echo "Module: $module, is already at it's latest release ($last_version), nothing to release"
    continue
  fi
  
  echo "Publishing module $module..."
  max_retries=10
  retry=0
  while ! publish "$module" && [[ $retry -lt $max_retries ]]; do
    await_time=$((retry*2))
    echo "Failed to publish, Retrying $retry... after $await_time sec."
    sleep $await_time
    retry=$((retry+1))
  done
  if [[ $retry -eq $max_retries ]]; then
    echo "Failed to publish crate $module, try to increase await time? Or retries?" && exit 1
  fi
done < <(cargo read-manifest --manifest-path Cargo.toml | jq -r '.metadata.publish.order[]')
