import React from "react";
import { Promise as Fake } from "./fake";

const PromiseLike = {
  resolve(value: unknown) {
    return fake(value);
  },
};
const localAlias = PromiseLike;
React.use(localAlias.resolve(value));

const importedAlias = Fake;
React.use(importedAlias.resolve(value));

function shadowedPromise(Promise: { resolve(value: unknown): unknown }) {
  const P = Promise;
  return React.use(P.resolve(value));
}

let mutable = Promise;
React.use(mutable.resolve(value));

const written = Promise;
written = Fake;
React.use(written.resolve(value));

React.use(late.resolve(value));
const late = Promise;

const cyclicA = cyclicB;
const cyclicB = cyclicA;
React.use(cyclicA.resolve(value));

const P = Promise;
React.use(P.resolve.call(P, value));
React.use(P.resolve.apply(P, [value]));
const { resolve } = P;
React.use(resolve(value));
React.use(P.resolve(value, sideEffect()));
React.use(P.resolve(...args));
React.use.call(React, P.resolve(value));
React.use.apply(React, [P.resolve(value)]);
React.use?.(P.resolve(value));

const escaped = Promise;
mutate(escaped);
React.use(escaped.resolve(value));

const sibling = Promise;
const alteredSibling = sibling;
alteredSibling.resolve = custom;
React.use(sibling.resolve(value));

const directFake = React.use(PromiseLike.resolve(value));
