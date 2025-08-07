#!/bin/bash

# get the version from the Cargo.toml file
version=$(cat Cargo.toml | grep -E "version = \"[0-9]+\.[0-9]+\.[0-9]+\"" | awk -F'"' '{print $2}')

# if version is not found, exit
if [ -z "$version" ]; then
    echo "Version not found in Cargo.toml"
    exit 1
fi

# if version is found, update it to the next minor version
cat Cargo.toml | sed -E "s/version = \"[0-9]+\.[0-9]+\.[0-9]+\"/version = \"$(echo $version | awk -F. '{print $1"."$2+1".0"}')/" > Cargo.toml.tmp
mv Cargo.toml.tmp Cargo.toml

# wait for user to review the change
read -p "Review the change and press Enter to continue..."

# commit the change
git commit -m "Bump version to $(echo $version | awk -F. '{print $1"."$2+1".0"}')"

# push the change
cargo publish