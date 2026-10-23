import React from "react";
const P = Promise;
const root = globalThis;
root.Promise.resolve = custom;
const value = React.use(P.resolve(pending));
