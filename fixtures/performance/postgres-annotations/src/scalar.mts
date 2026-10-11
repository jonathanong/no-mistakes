// Keep scalar snapshots independent: overlapping nested joins obscure reachability cost.
export function scalarPayload(flag: boolean) {
  const scalar00 = flag
    ? ["branch-00", ["active", 1, true], ["columns", "id", "status"]]
    : ["fallback-00", ["inactive", 0, false], ["columns", "id", "created_at"]];
  const scalar01 = flag
    ? ["branch-01", ["active", 1, true], ["columns", "id", "status"]]
    : ["fallback-01", ["inactive", 0, false], ["columns", "id", "created_at"]];
  const scalar02 = flag
    ? ["branch-02", ["active", 1, true], ["columns", "id", "status"]]
    : ["fallback-02", ["inactive", 0, false], ["columns", "id", "created_at"]];
  const scalar03 = flag
    ? ["branch-03", ["active", 1, true], ["columns", "id", "status"]]
    : ["fallback-03", ["inactive", 0, false], ["columns", "id", "created_at"]];
  return [scalar00, scalar01, scalar02, scalar03];
}
