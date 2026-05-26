import { vi } from "vitest";

helpers.fn();
function useLocal(vi) {
  vi.mock("./local");
}
