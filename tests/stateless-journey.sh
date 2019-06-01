#!/usr/bin/env bash
set -eu

exe=${1:?First argument must be the executable to test}

root="$(cd "${0%/*}" && pwd)"
exe="$root/../$exe"
# shellcheck disable=1090
source "$root/utilities.sh"
snapshot="$root/snapshots"
fixtures="$root/fixtures"

SUCCESSFULLY=0

(with "a sample directory"
  (sandbox
    cp -R "$fixtures/sample-01" .
    (with "no arguments"
      it "produces a human-readable (metric) aggregate of the current directory, without total" && {
        WITH_SNAPSHOT="$snapshot/success-no-arguments" \
        expect_run ${SUCCESSFULLY} "$exe"
      }
    )

    (with "the byte format set"
      (with "human-binary"
        it "produces a human-readable aggregate of the current directory, without total" && {
          WITH_SNAPSHOT="$snapshot/success-bytes-binary" \
          expect_run ${SUCCESSFULLY} "$exe" --format humanbinary
        }
      )
      (with "bytes"
        it "produces a human-readable aggregate of the current directory, without total" && {
          WITH_SNAPSHOT="$snapshot/success-bytes-only" \
          expect_run ${SUCCESSFULLY} "$exe" --format bytes
        }
      )
    )
  )
)
