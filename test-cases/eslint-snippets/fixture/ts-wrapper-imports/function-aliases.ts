import * as users from "@example/users";

const m = require("@example/mod");
import n = require("@example/mod");

function createUser(x) {
  return x;
}

export const orig = ((x) => x) satisfies (value: string) => string;

export function wrap(a) {
  return m.run(a);
}

export function wrapCall(a) {
  return n.call(a);
}

export function viaComputed(x) {
  return users["createUser" as const](x);
}

export function viaApply(x) {
  return users.createUser.apply(users, [x]!);
}

export function viaCast(x) {
  return (orig as typeof orig)(x);
}

export const aliased = ((x) => createUser(x))!;
