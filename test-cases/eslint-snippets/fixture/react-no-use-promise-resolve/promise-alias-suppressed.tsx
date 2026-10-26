import React from "react";
const P = Promise;
// no-mistakes-disable-next-line react-no-use-promise-resolve -- retained compatibility probe
const nextLine = React.use(P.resolve(value));
const line = React.use(P.resolve(value)); // no-mistakes-disable-line react-no-use-promise-resolve -- retained compatibility probe
