import React from "react";
const P = Promise;
delete P.resolve;
const value = React.use(P.resolve(pending));
