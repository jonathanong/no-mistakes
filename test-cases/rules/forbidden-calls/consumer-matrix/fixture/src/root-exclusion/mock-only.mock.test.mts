import { mockOnly } from "./mock-only.mts";

export function runMockOnlyRoot() {
  setTimeout(() => {}, 1);
  mockOnly();
}
