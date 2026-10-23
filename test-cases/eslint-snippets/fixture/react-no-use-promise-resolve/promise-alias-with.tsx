import React from "react";
// Deliberately invalid ESM: TypeScript ESLint accepts this AST for the defensive scope guard.
const P = Promise;
with (
  {
    get P() {
      return Fake;
    },
  }
) {
  React.use(P.resolve(value));
}
with (
  {
    get Promise() {
      return Fake;
    },
  }
) {
  const local = Promise;
  React.use(local.resolve(value));
}
