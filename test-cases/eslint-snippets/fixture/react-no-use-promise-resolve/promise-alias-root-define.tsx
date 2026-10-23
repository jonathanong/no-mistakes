import React from "react";
const P = Promise;
Object.defineProperty(Promise, "resolve", { value: custom });
const value = React.use(P.resolve(pending));
