import { p5 } from "@example/db";

// A parameter callback is not declared elsewhere, so the call stays Dynamic.
function run(cb: (x: { query: unknown }) => void, ..._rest: unknown[]) {
  cb(p5);
}

export function a() {
  return p5.query("SELECT id FROM users WHERE x = 1");
}

export function b() {
  run((x) => {
    x.query = () => undefined;
  });
}
