#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Push no compiled dependency trees to Cachix",
  status: done(2026, 10, 29),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

CI pushed every `-deps-` build, hundreds of MB each, into the free 5 GB mlavrinenko cache it shares with Mindtape, evicting the binaries consumers fetch.
