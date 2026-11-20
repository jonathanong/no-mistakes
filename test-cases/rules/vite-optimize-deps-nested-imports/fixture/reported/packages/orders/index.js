import { shape } from "items";
export { shape as orderShape } from "items";
export * from "items";

const loaded = require("items");
const again = require(`items`);
const name = "items";
require(name);

export const order = loaded ?? again ?? shape;
