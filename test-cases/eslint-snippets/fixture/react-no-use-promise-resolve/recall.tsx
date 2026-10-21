import React from "react";
import { use as useAlias } from "react";
import * as ReactNs from "react";

declare const method: string;

export function View() {
  const { use } = React;
  const local = useAlias;
  const { use: taken } = ReactNs;
  useAlias(Promise["resolve"](1));
  local(Promise[`resolve`](1));
  use((Promise?.resolve)(1));
  (React?.use)(Promise.resolve(1));
  ReactNs["use"](Promise.resolve(1));
  React.use((0, Promise.resolve)(1));
  React.use(Promise.resolve.call(Promise, 1));
  taken(Promise.resolve(1));
  React.use.call(React, Promise.resolve(1));
  React.use.apply(React, [Promise.resolve(1)]);
  React.use["call"](React, Promise.resolve(1));
  React.use["apply"](React, [Promise.resolve(1)]);
  const { read } = React;
  read(Promise.resolve(1));
  React[method](Promise.resolve(1));
  // no-mistakes-disable-next-line react-no-use-promise-resolve -- retained probe
  React.use(Promise.resolve(1));
}
