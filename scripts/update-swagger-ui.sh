#!/bin/bash
#
# Update Swagger UI version

set -eu -o pipefail

version="${1:-""}"
if [ -z "$version" ]; then
    echo "Missing 'version' argument from command, run as $0 <version>" >&2 && exit 1
fi
zip_name="v$version.zip"

curl -sSL -o "$zip_name" "https://github.com/swagger-api/swagger-ui/archive/refs/tags/v$version.zip"

echo "Update vendored Swagger UI"
mv "$zip_name" ./utoipa-swagger-ui-vendored/res/
sed -i "s|version: \`.*\`|version: \`$version\`|" ./utoipa-swagger-ui-vendored/README.md
sed -i "s|version: \`.*\`|version: \`$version\`|" ./utoipa-swagger-ui-vendored/src/lib.rs
sed -i "s|res/v.*\.zip|res/v$version.zip|" ./utoipa-swagger-ui-vendored/src/lib.rs

echo "Update utoipa-swagger-ui Swagger UI version"
sed -i "s|tags/v.*>|tags/v$version.zip>|" ./utoipa-swagger-ui/README.md
sed -i "s|tags/v.*>|tags/v$version.zip>|" ./utoipa-swagger-ui/src/lib.rs
sed -i "s|tags/v.*\.zip|tags/v$version.zip|" ./utoipa-swagger-ui/build.rs
