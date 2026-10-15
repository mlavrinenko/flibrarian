#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Make the prose slots of the templates raw",
  kind: "fix",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling", "found"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      instant-search: neutral,
      any-fb2: neutral,
      safe-index: neutral,
      one-core: neutral,
      usable-files: neutral,
    ),
    process: (
      sound-codebase: helps[a code span typed into a template slot files as a code span],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 15),
)

== Problem

+ Run `mt add -t tweak --describe` or `mt add -t finding --describe` and read the slot table. The slots `why`, `context` and `observation` are `kind: "text"`, and their examples show code spans.
+ 🔴 `mt add` escapes a `text` slot, so a code span typed there is filed as a backslash before each backtick and each `<`, `*`, `#` or `@` inside it. No filed task carries that damage yet, but the next tweak or finding that follows the examples will.

== Environment

- `mt` 0.3.0 on Linux, in the `flibrarian` checkout at its `main` branch.

== TO-BE

- 🟢 The prose slots of the tweak and finding templates are `kind: "raw"`, so a code span typed into them files as a code span.

== Changes

- `.mindtape/templates/tweak.typ`: `why` becomes `kind: "raw"`.
- `.mindtape/templates/finding.typ`: `context` and `observation` become `kind: "raw"`.
- `mindtape` commits 98ce1e44 and 64dfa4f9 show the same fix in its own repo.

== Out of scope

- Slots that hold a number or a short label, such as `estimate`. Escaping leaves those unchanged.

== Dogfood

Not applicable. The change repairs this repo's own templates and task files.
