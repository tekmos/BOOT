#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

function print_usage() {
  echo "Usage: $0 [-h|--help]"
  echo "Publishes crates to crates.io."
}

if [ $# = 1 ] && { [ "$1" = "-h" ] || [ "$1" = "--help" ] ; }
then
    print_usage
    exit 1
fi

# these are imported by other packages
BASE_PACKAGES="boot-contract-derive boot-fns-derive"
UTILS_PACKAGES="boot-core"
ALL_PACKAGES="boot-cw-plus"

for pack in $BASE_PACKAGES; do
  (
    cd "packages/$pack"
    echo "Publishing $pack"
    cargo publish
  )
done

for pack in $UTILS_PACKAGES; do
  (
    cd "$pack"
    echo "Publishing $pack"
    cargo publish
  )
done

for pack in $ALL_PACKAGES; do
  (
    cd "$pack"
    echo "Publishing $pack"
    cargo publish
  )
done

echo "Everything is published!"

VERSION=$(< Cargo.toml grep -m 1 version | sed 's/-/_/g' | grep -o '".*"' | sed 's/"//g');
git tag v"$VERSION"
git push origin v"$VERSION"