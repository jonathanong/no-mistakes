import React from "react";
Promise.patch = function () {
  this.resolve = custom;
};
Promise.patch();
const P = Promise;
const value = React.use(P.resolve(pending));
