import React, { use as read } from "react";

const P = Promise;
const scalar = React.use(P.resolve(initialValue));
const pending = loadValue();
const existing = React.use(P.resolve(pending));

const Q = P;
const chained = React.use(Q.resolve(initialValue));

const one = Promise,
  two = one;
const sameStatement = read(two.resolve(initialValue));

const stringKey = React.use(P["resolve"](initialValue));
const templateKey = React.use(P[`resolve`](initialValue));

function nestedAlias(value: string) {
  const Local = Promise;
  return React.use(Local.resolve(value));
}

const renamed = read(P.resolve(initialValue));

Promise.all([]);
new Promise((resolve) => resolve(initialValue));

type PromiseValue = Promise<string>;
const Typed = Promise;
const typedValue = React.use(Typed.resolve("ready"));
