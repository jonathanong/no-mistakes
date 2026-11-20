export type * from "labels";

function label() {}
label();

const box = {
  ready() {},
};
box.ready();

import(`items`);
import(("items"));
require(("items"));

export const ready = true;
