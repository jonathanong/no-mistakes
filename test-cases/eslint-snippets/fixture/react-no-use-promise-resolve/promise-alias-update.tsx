import React from "react";
const P = Promise;
P.resolve++;
const value = React.use(P.resolve(pending));
