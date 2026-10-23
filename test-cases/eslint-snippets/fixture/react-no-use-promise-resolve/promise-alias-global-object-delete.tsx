import React from "react";
const P = Promise;
delete globalThis.other;
const result = React.use(P.resolve(pending));
