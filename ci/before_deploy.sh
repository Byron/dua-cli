# This script takes care of building your crate and packaging it for release

set -ex

main() {
  # We run tests only once, and won't try to create any additional artifacts
  if [ -n "$ENABLE_TESTS" ]; then
    return
  fi
  
    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    cross rustc --bin $CRATE_NAME --target $TARGET --release -- -C lto

    strip target/$TARGET/release/$CRATE_NAME
    cp target/$TARGET/release/$CRATE_NAME $stage/

    cd $stage
    tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
    cd $src

    rm -rf $stage
}

main
