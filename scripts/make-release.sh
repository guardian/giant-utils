set -e

pushd () {
    command pushd "$@" > /dev/null
}

popd () {
    command popd "$@" > /dev/null
}

SCRIPT_PATH=$( cd $(dirname $0) ; pwd -P )

rustup target add aarch64-apple-darwin x86_64-apple-darwin

pushd "$SCRIPT_PATH/.."

VERSION=$(grep '^version =' Cargo.toml | sed 's/version = //g' | sed 's/"//g')

ALL_TRIPLES=$(rustup target list --installed)

for TRIPLE in $ALL_TRIPLES; do
    echo ''
    echo "=== Creating release for $TRIPLE ==="
    cargo build --release --quiet --target="$TRIPLE"

    pushd "target/$TRIPLE/release"

    GIANT_TAR_NAME="giant-utils_${TRIPLE}_v${VERSION}.tar.gz"

    tar -czf "$GIANT_TAR_NAME" giant-utils

    echo "The following is the SHA256 sum for the '$GIANT_TAR_NAME' bundle:"
    shasum -a 256 "$GIANT_TAR_NAME"

    popd
done

popd