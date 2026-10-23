import React from "react";
const P = Promise;
window.Promise.resolve = custom;
const value = React.use(P.resolve(pending));
