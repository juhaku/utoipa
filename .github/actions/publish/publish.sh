#!/bin/bash

# Publishes crate to crates.io

token=""
while true; do
  case $1 in
    "--token")
      shift
      token="$1"
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

function publish {
  module="$1"
  if [[ "$module" == "utoipa" ]]; then
    cargo publish
  else
    cargo publish -p "$module"
  fi
}

echo "$token" | cargo login
while read -r manifest; do
  echo "Publishing module $manifest..."
  module=$(cargo read-manifest --manifest-path "$manifest" | jq -r .name)
  
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
done < <(find . ! -path "*target*" -name "Cargo.toml" | sort -r)
