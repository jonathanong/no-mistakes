import * as v from "vitest";
v.test("reads", () => {
  const err = new Error("boom");
  if (err.message === ("boom" as string)) return;
  (v as typeof v).expect(err.message).toBe("boom");
  v.expect.apply(null, [err.message] as const).toBe("boom");
  (v.expect<string>)(err.message).toBe("boom");
});
