#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: slot(
    "title",
    prompt: "Task title, imperative verb first",
    example: "Test that a UTF-16 FB2 indexes like any other",
  ),
  priority: ice(
    impact: slot("impact", kind: "int", default: 5),
    confidence: slot("confidence", kind: "float", default: 0.8),
    ease: slot("ease", kind: "int", default: 5),
  ),
  difficulty: slot("difficulty", kind: "int", default: 3),
  kind: slot("kind", example: "test"),
  tags: slot("tags", kind: "list", default: ()),
  estimate: slot("estimate", kind: "text", default: "(src-files: 2)"),
  x-goals: (
    product: (
      instant-search: slot("instant-search", kind: "raw", default: "neutral"),
      any-fb2: slot(
        "any-fb2",
        kind: "raw",
        default: "neutral",
        example: "helps[a UTF-16 archive no longer aborts the run]",
      ),
      safe-index: slot("safe-index", kind: "raw", default: "neutral"),
      one-core: slot("one-core", kind: "raw", default: "neutral"),
      usable-files: slot("usable-files", kind: "raw", default: "neutral"),
    ),
    process: (
      sound-codebase: slot("sound-codebase", kind: "raw", default: "neutral"),
      docs-match-code: slot("docs-match-code", kind: "raw", default: "neutral"),
    ),
  ),
)

== Problem

#slot(
  "problem",
  kind: "raw",
  prompt: "Numbered steps anyone can follow, commands in backticks; open the step that shows the defect with 🔴 and quote what it printed",
  example: ```typ
  + Run `flibrarian index ./mylib` on an archive holding a UTF-16 FB2.
  + 🔴 The run prints `invalid utf-8` and skips the book.
  ```,
)

== TO-BE

#slot(
  "to-be",
  kind: "raw",
  prompt: "Bullets opening with 🟢: how it should work, behave and feel once this lands",
  example: ```typ
  - 🟢 A UTF-16 FB2 indexes like any other, and the log names the re-encoded book once.
  ```,
)

== Changes

#slot(
  "changes",
  kind: "raw",
  prompt: "One bullet per file: `path`, then what changes there; the red test first",
  example: ```typ
  - `crates/flibrarian-core/tests/encoding_test.rs`: the red test first, a UTF-16 FB2 indexing.
  - `crates/flibrarian-core/src/encoding.rs`: decode UTF-16 before parsing.
  ```,
)

== Out of scope

#slot(
  "out-of-scope",
  kind: "raw",
  prompt: "Bullets: what not to touch, and why",
  example: ```typ
  - Legacy code pages. Each gets its own task once someone hits one.
  ```,
)

== Dogfood

#slot(
  "dogfood",
  kind: "raw",
  prompt: "How this repo will use the change, or Not applicable and why",
  example: ```typ
  The maintainer's own library indexes without a skipped book.
  ```,
)
