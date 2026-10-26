// Module-scope runner fixtures retain their established test-body meaning.
const test = makeRunner();
let it = makeRunner();
test("const runner", () => { const match = err.message === "x"; });
it("mutable named runner", () => { const match = err.message === "x"; });
