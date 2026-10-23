import React from "react";
const P = Promise;
globalThis.Promise.resolve = custom;
const value = React.use(P.resolve(pending));
