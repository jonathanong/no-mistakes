// no-mistakes-disable-file react-no-use-promise-resolve -- retained compatibility probe
import React from "react";
const P = Promise;
const file = React.use(P.resolve(value));
