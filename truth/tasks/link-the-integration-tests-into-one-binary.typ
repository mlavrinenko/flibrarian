#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Link the integration tests into one binary",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tests", "core"),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      instant-search: neutral,
      any-fb2: neutral,
      safe-index: neutral,
      one-core: neutral,
      usable-files: neutral,
    ),
    process: (
      sound-codebase: helps[one test binary links once, so `just check` and `just coverage` build less and the target dir stays small],
      docs-match-code: helps[CONTRIBUTING says where a new test file goes],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
)

== Problem

+ Run `cargo test -p flibrarian-core --no-run`. Cargo links every dependency again for each file in `crates/flibrarian-core/tests/`, the only crate with a `tests/` directory.
+ 🔴 It builds 22 integration executables of about 29 MB each (639 MB, measured 2026-10-11), and `extraction_test.rs` and `integration_test.rs` each compile their own copy of `tests/common/mod.rs`.
+ For scale: mindtape's 74 test files gave 263 executables averaging 298 MB in `target/debug/deps` (80 GB), and `just cover` doubled that (51 GB in one worktree; the disk filled). Here `just coverage` (tarpaulin) rebuilds the 22 too.


== TO-BE

- 🟢 `cargo test --workspace --no-run 2>&1 | grep -c Executable` drops from 26 to 5: one integration executable, `it`, plus the 4 unit executables.
- 🟢 Test totals match the baseline: 296 for `cargo test --workspace --quiet`, 291 for `-p flibrarian-core` (the workspace run enables `faking` through `flibrarian-cli`).
- 🟢 Tests now run as threads of one process. No file sets an env var, changes the cwd, binds a port or uses a `static` (grepped 2026-10-11), so none should clash. Run the suite several times: a test failing only beside others is a latent bug to fix, not a reason to revert.
- 🟢 CONTRIBUTING.md says a new test is `tests/it/<name>.rs` plus a `mod` line.


== Changes

- Baseline first: both totals and the executable count (26). Finish with `just check` green and `just coverage` running.
- `crates/flibrarian-core/tests/`: `git mv *.rs it/` and `git mv common it/common`. Leave `res/`: fixtures open as `tests/res/<name>` from the crate dir, which cargo still uses as cwd.
- `crates/flibrarian-core/tests/it/main.rs`: new, `mod common;` plus one `mod <stem>;` per moved file, names unchanged. Cargo autodiscovers target `it`; add no `[[test]]` entry.
- `crates/flibrarian-core/tests/it/extraction_test.rs`, `integration_test.rs`: drop `mod common;`, write `use crate::common::{..}`. Leave the 10 `flibrarian_core::common` imports: that is the library.
- `include_str!` resolves against its file, so add one `../` in `extension_policy_test.rs:18`, `searching_coverage_test.rs:12`, `search_index_test.rs:10`, `crash_resilience_test.rs:51`, `covers_annotations_test.rs:14` and `common/mod.rs:48`.
- Keep `#![cfg(feature = "faking")]` (`crash_resilience_large_test.rs:1`) and `#![allow(dead_code)]` (`common/mod.rs:1`) first in their files; test with and without `--features faking`.
- `.linecop.yaml`: `**/tests/*.rs` (limit 400) becomes `**/tests/**/*.rs`, or `indexing_test.rs` (396 lines) falls to the 300 cap.
- `CONTRIBUTING.md`: the Russian "Тесты" paragraph gets the new-file rule.


== Rollback

- Revert the commit. It moves files and edits paths only.


== Out of scope

- Inline `#[cfg(test)]` unit tests: they already share one binary per crate.
- Debug info and linker settings: `[profile.test]` already uses line tables.
- Renaming test files or splitting `indexing_test.rs`.
- Already correct: the Justfile (`test`, `check-rust`, `check-compile`, `coverage`), `tarpaulin.toml`, `flake.nix:153` and `release.yml:82` pass no `--test` or test file, and there is no nextest, mutants or `[[test]]` config.


== Dogfood

The next `just check` and `just coverage` in this repo link one test executable.

