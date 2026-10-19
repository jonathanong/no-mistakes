import { p3 } from "@example/db";

// A later assignment is not an ambient declaration, so the call stays Dynamic.
let g: (x: { query: unknown }) => void;
g = (x) => {
  x.query = () => undefined;
};

export function a() {
  return p3.query("SELECT id FROM users WHERE x = 1");
}

export function b() {
  g(p3);
}
