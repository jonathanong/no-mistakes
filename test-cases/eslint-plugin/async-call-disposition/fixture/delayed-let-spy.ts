import { enqueueEmail as original, enqueueRelease } from "@app/jobs";
import { vi, type MockInstance } from "vitest";

// The optional annotation is metadata; the single assignment proves provenance.
function guarded(tags, p, x, admitted, release, m) {
  let spy: MockInstance<typeof m.enqueue> | undefined;
  try {
    spy = vi.spyOn(m, "enqueue");
    spy.mockImplementation((tags, p) => {
      if (!tags.includes(x)) return original(tags, p);
      admitted.resolve();
      return release.promise;
    });
  } finally {
    spy?.mockRestore();
  }
}

function blocked(m) {
  let spy;
  {
    spy = vi.spyOn(m, "enqueue");
    spy.mockImplementationOnce((tags, p) => original(tags, p));
  }
}

// Both configured promises pass through the same established mock.
function twoReturns(m, selected) {
  let spy;
  spy = vi.spyOn(m, "enqueue");
  spy.mockImplementation((tags, p) => {
    if (selected) return original(tags, p);
    return enqueueRelease(tags, p);
  });
}
