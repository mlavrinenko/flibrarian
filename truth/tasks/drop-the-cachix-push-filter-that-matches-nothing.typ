#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Drop the Cachix push filter that matches nothing",
  status: done(2026, 10, 29),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

The `-deps-` filter from 9d01137 matches nothing: `buildRustPackage` makes no dependency-tree path, and CI pushes only flibrarian, flibrarian-web and flibrarian-frontend.
