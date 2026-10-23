import React from "react";
const P = Promise;
const Q = P;
Q.resolve = custom;
const value = React.use(P.resolve(pending));
