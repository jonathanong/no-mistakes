// Kept on disk to prove transitive fetch traversal cannot re-enter ignored helpers.
export function ignoredFetch() {
  return fetch("/api/ignored-helper");
}
