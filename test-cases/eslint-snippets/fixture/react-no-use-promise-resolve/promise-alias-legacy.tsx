import React from "react";
const Promise = {
  resolve(value: unknown) {
    return fake(value);
  },
};
React.use(Promise.resolve(value));
React.use(Promise.resolve.call(Promise, value));
