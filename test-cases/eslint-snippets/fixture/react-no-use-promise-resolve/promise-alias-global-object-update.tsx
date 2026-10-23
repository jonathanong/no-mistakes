import React from "react";
const P = Promise;
++globalThis.count;
const result = React.use(P.resolve(pending));
