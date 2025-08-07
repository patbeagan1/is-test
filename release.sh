#!/usr/bin/env zsh

# read the version from the cli 
read "version?Version?"
echo "$version"

# if version is found, update it to the next minor version
sed -i -E "s/^version = \"[0-9]+\.[0-9]+\.[0-9]+\"\$/version = \"$(echo $version)\"/" Cargo.toml

# wait for user to review the change
read "devnull?Review the change and press Enter to continue..."

cargo build

# commit the change
git add -A
git commit -am "Bump version to $(echo $version | awk -F. '{print $1"."$2+1".0"}')"

# push the change
cargo publish