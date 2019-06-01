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
WITH_FAILURE=1

(with "a sample directory"
  (sandbox
    cp -R "$fixtures/sample-01/" .
    (with "no arguments"
      (with "no given path"
        it "produces a human-readable (metric) aggregate of the current directory, without total" && {
          WITH_SNAPSHOT="$snapshot/success-no-arguments" \
          expect_run ${SUCCESSFULLY} "$exe"
        }
      )
      ls
      (with "multiple given paths"
        (when "specifying the 'aggregate' subcommand"
          (with "no option to adjust the total"
            it "produces a human-readable (metric) aggregate of the current directory, with total" && {
              WITH_SNAPSHOT="$snapshot/success-no-arguments-multiple-input-paths" \
              expect_run ${SUCCESSFULLY} "$exe" aggregate . . dir ./dir/ ./dir/sub
            }
          )
          (with "the --no-total option set"
            it "produces a human-readable (metric) aggregate of the current directory, without total" && {
              WITH_SNAPSHOT="$snapshot/success-no-arguments-multiple-input-paths-no-total" \
              expect_run ${SUCCESSFULLY} "$exe" aggregate --no-total . . dir ./dir/ ./dir/sub
            }
          )
        )
        (when "specifying no subcommand"
          it "produces a human-readable (metric) aggregate of the current directory, with total" && {
            WITH_SNAPSHOT="$snapshot/success-no-arguments-multiple-input-paths" \
            expect_run ${SUCCESSFULLY} "$exe" . . dir ./dir/ ./dir/sub
          }
        )
        (when "specifying no subcommand and some of the directories don't exist"
          it "produces a human-readable (metric) aggregate of the current directory, with total" && {
            WITH_SNAPSHOT="$snapshot/failure-no-arguments-multiple-input-paths-some-not-existing" \
            expect_run ${WITH_FAILURE} "$exe" . . foo bar baz
          }
        )
      )
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
