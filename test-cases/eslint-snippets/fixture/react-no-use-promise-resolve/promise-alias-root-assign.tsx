import React from "react";
const P = Promise;
Object.assign(Promise, { resolve: custom });
const value = React.use(P.resolve(pending));
