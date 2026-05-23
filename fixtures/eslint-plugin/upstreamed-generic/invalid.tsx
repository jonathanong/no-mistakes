import React, { use } from "react";
import { use as otherUse } from "not-react";

const values = [3, 1, 2];
await values.sort();
delete values.length;
export type Placeholder = never;
type OtherPlaceholder = never;
export type { OtherPlaceholder };
let counter = 0;
it("mutates shared state", () => {
  counter++;
});
expect(error.message).toContain("missing");
error.message.includes("missing");
if (error.message === "missing") {
  throw error;
}
test.sequential("runs", () => {});
React.use(Promise.resolve("ok"));
use(Promise.resolve("ok"));
otherUse(Promise.resolve("ok"));

export default function Component() {
  return (
    <>
      <script src="/x.js" />
      <div>{(() => "inline")()}</div>
    </>
  );
}
