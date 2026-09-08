import { shared } from "./diamond-shared.mts";

export function importedButNotCalled() {
  return shared;
}
