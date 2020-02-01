# This script takes care of testing your crate

set -ex

main() {
  if [ -n "$ENABLE_TESTS" ]; then
    # for now, we can only run unit-tests until it's clear why journey-test results differ on Linux
    # TODO: Test in docker, make sure we get the same results
    make unit-tests
    return
  fi
  cross build --target $TARGET
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
  main
fi
