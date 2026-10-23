import React from "react";
const P = Promise;
const root = globalThis;
const child = root;
Object.defineProperty(child.Promise, "resolve", { value: custom });
const value = React.use(P.resolve(pending));
