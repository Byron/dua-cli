#!/usr/bin/env bash
set -eu

exe=${1:?First argument must be the executable to test}

root="$(cd "${0%/*}" && pwd)"
# shellcheck disable=1090
source "$root/utilities.sh"
snapshot="$root/snapshots"

SUCCESSFULLY=0
WITH_FAILURE=1

(with "no input file"
  it "fails with an error message" && {
    WITH_SNAPSHOT="$snapshot/failure-missing-input-file" \
    expect_run ${WITH_FAILURE} "$exe"
  }
)
(with "a valid input file"
  it "produces the expected output" && {
    WITH_SNAPSHOT="$snapshot/success-input-file-produces-correct-output" \
    expect_run ${SUCCESSFULLY} "$exe" <(echo this is probably not what you want)
  }
)
