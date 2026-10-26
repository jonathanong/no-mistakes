import React from "react";
const P = Promise;
const root = window;
root.Promise.resolve = custom;
const value = React.use(P.resolve(pending));
