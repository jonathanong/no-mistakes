import React from "react";

const values = [3, 1, 2];
values.sort();
await query.sort();
const copy = { a: 1, b: 2 };
const { b, ...withoutB } = copy;
export type RealType = { id: string };
it("keeps state local", () => {
  let counter = 0;
  counter++;
});
let shadow = 0;
it("allows local shadowing", () => {
  let shadow = 0;
  shadow++;
});
expect(error.code).toBe("ENOENT");
expect(response.body.message).toBe("created");
expect(result.body?.message).toContain("ok");
let sharedFixture = null;
describe("setup fixtures", () => {
  beforeAll(() => {
    sharedFixture = createFixture();
  });
  afterEach(() => {
    sharedFixture = null;
  });
  it("uses setup fixture", () => {
    console.log(sharedFixture);
  });
});
let mockPathname = "/";
vi.mock("next/navigation", () => ({ usePathname: () => mockPathname }));
let mockdata = {};
vi.mock("shared/data", () => ({ data: mockdata }));
it("uses mock control", () => {
  mockPathname = "/next";
  mockdata = { next: true };
});
test.parallel("runs", () => {});
const promise = Promise.resolve("ok");
React.use(promise);
const use = "memo";
React[use](Promise.resolve("ok"));
export const metadata = {};
export default function Page() {
  const label = "Save";
  return (
    <>
      <button data-pw="save">{label}</button>
      <span>{label}</span>
    </>
  );
}
