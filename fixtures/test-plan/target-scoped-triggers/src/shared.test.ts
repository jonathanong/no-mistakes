import { describe, expect, it } from "vitest";
import { pick } from "lodash";

// Both projects intentionally own this file so targeted command filtering is observable.
describe("shared", () => {
  it("keeps only selected execution targets", () =>
    expect(pick({ selected: true }, ["selected"])).toEqual({ selected: true }));
});
