import React from "react";
const P = Promise;
global.Promise.resolve = custom;
const value = React.use(P.resolve(pending));
