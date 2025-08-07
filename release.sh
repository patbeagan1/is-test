#!/bin/bash

# read the version from the cli 
read -p "Version?" version

# if version is found, update it to the next minor version
sed -i -E "s/version = \"[0-9]+\.[0-9]+\.[0-9]+\"/version = \"$(echo $version)/" Cargo.toml

# wait for user to review the change
read -p "Review the change and press Enter to continue..."

# commit the change
git commit -m "Bump version to $(echo $version | awk -F. '{print $1"."$2+1".0"}')"

# push the change
cargo publish