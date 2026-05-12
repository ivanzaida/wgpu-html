// Integration-style tests need a real font file. The cache shape
// (sync invariants, idempotence on Arc identity) doesn't, so we
// keep those in `lui-tree` where there are no font deps.
// Demo + downstream layout tests cover the real shaping paths.
