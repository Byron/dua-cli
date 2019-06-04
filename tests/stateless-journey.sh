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
        (with "no subcommand"
          it "produces a human-readable (metric) aggregate of everything within the current directory, with total" && {
            WITH_SNAPSHOT="$snapshot/success-no-arguments" \
            expect_run ${SUCCESSFULLY} "$exe"
          }
        )
        (with "the aggregate sub-command"
          (with "no sorting option"
            it "produces a human-readable (metric) aggregate of everything within the current directory, with total" && {
              WITH_SNAPSHOT="$snapshot/success-no-arguments" \
              expect_run ${SUCCESSFULLY} "$exe" aggregate
            }
          )
          (with "sorting disabled"
            it "produces a human-readable (metric) aggregate of everything within the current directory, alphabetically sorted, with total" && {
              WITH_SNAPSHOT="$snapshot/success-no-arguments-no-sort" \
              expect_run ${SUCCESSFULLY} "$exe" aggregate --no-sort
            }
          )
        )
      )
      (with "multiple given paths"
        (when "specifying the 'aggregate' subcommand"
          (with "no option to adjust the total"
            it "produces a human-readable aggregate, with total" && {
              WITH_SNAPSHOT="$snapshot/success-no-arguments-multiple-input-paths" \
              expect_run ${SUCCESSFULLY} "$exe" a . . dir ./dir/ ./dir/sub
            }
          )
          (with "the --no-total option set"
            it "produces a human-readable aggregate, without total" && {
              WITH_SNAPSHOT="$snapshot/success-no-arguments-multiple-input-paths-no-total" \
              expect_run ${SUCCESSFULLY} "$exe" aggregate --no-total . . dir ./dir/ ./dir/sub
            }
          )
          (with "the --no-sort option set"
            it "produces a human-readable aggregate, sorted in order specified on the command-line" && {
              WITH_SNAPSHOT="$snapshot/success-no-arguments-multiple-input-paths-no-sort" \
              expect_run ${SUCCESSFULLY} "$exe" aggregate --no-sort . . dir ./dir/ ./dir/sub
            }
          )
          (with "the --stats option set"
            it "produces a human-readable aggregate, and statistics about the iteration in RON" && {
              WITH_SNAPSHOT="$snapshot/success-no-arguments-multiple-input-paths-statistics" \
              expect_run ${SUCCESSFULLY} "$exe" aggregate --stats . . dir ./dir/ ./dir/sub
            }
          )
        )
        (when "specifying no subcommand"
          it "produces a human-readable aggregate" && {
            WITH_SNAPSHOT="$snapshot/success-no-arguments-multiple-input-paths" \
            expect_run ${SUCCESSFULLY} "$exe" . . dir ./dir/ ./dir/sub
          }
        )
        (when "specifying no subcommand and some of the directories don't exist"
          it "produces a human-readable aggregate, with the number of errors per root" && {
            WITH_SNAPSHOT="$snapshot/failure-no-arguments-multiple-input-paths-some-not-existing" \
            expect_run ${WITH_FAILURE} "$exe" . . foo bar baz
          }
        )
      )
    )

    (with "the byte format set"
      for format in binary bytes metric gb gib mb mib; do
        (with $format
          it "produces a human-readable aggregate of the current directory, without total" && {
            WITH_SNAPSHOT="$snapshot/success-bytes-$format" \
            expect_run ${SUCCESSFULLY} "$exe" --format $format
          }
        )
      done
    )
  )
  (with "interactive mode"
    it "fails as there is no TTY connected" && {
      WITH_SNAPSHOT="$snapshot/failure-interactive-without-tty" \
      expect_run ${WITH_FAILURE} "$exe" i
    }
  )
)
