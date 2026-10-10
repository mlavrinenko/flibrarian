// DRAFT for the maintainer to review: these goals were derived from the
// README and the code, not set by you. Reword, drop or add before relying on
// them; every open task answers each one, so a bad goal costs on every filing.
//
// What flibrarian is for. Every open task gives each goal a verdict in its
// `x-goals: (product: ...)` field, checked by `check-goals(...)` from
// truth/lib.typ. How this repo builds it is a separate set, in
// process-goals.typ.

#let goals = (
  instant-search: (
    title: "Search answers as you type",
    acid-test: [A full-text query over a fully indexed library lists its
      matches before the user finishes the next keystroke.],
  ),
  any-fb2: (
    title: "Broken FB2 still lands in the library",
    acid-test: [An archive with malformed XML, a UTF-16 or legacy encoding, or
      missing fields yields a searchable entry, never an aborted run.],
  ),
  safe-index: (
    title: "A crash never costs the index",
    acid-test: [Kill the indexer at any point; the next run resumes from a
      usable index, and an unwritable library is reported, not hung on.],
  ),
  one-core: (
    title: "One core, three faces",
    acid-test: [Index, search and extract behave the same from the CLI, the
      web server and the desktop app, because all three call flibrarian-core.],
  ),
  usable-files: (
    title: "Extracted books open anywhere",
    acid-test: [Every extracted file gets a name valid on the common
      filesystems, within 255 bytes, built from the author and the title.],
  ),
)
