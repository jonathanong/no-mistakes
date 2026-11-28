import { expect } from "vitest";

interface User {
  name: string;
  nickname?: string;
  profile: { email: string };
  tags?: string[];
}
declare function loadUser(): User | undefined;
declare function loadResult(): { item?: { status: string } };
declare function loadHandler(): (() => number) | undefined;
declare const unrelatedName: string;

export async function definedThenDeref() {
  // Positive cases (defined-then-deref)
  const user = loadUser();
  expect(user).toBeDefined();
  expect(user!.name).toBe("Alice");

  const maybeUser = loadUser();
  expect(maybeUser).toBeDefined();
  expect(maybeUser?.name).toBe("Alice");

  const notUndefUser = loadUser();
  expect(notUndefUser).not.toBeUndefined();
  expect(notUndefUser!.profile.email).toContain("@");

  const result = loadResult();
  expect(result.item).toBeDefined();
  expect(result.item?.status).toBe("ok");

  const handler = loadHandler();
  expect(handler).toBeDefined();
  expect(handler!()).toBe(1);

  const optHandler = loadHandler();
  expect(optHandler).toBeDefined();
  expect(optHandler?.()).toBe(1);

  const aliased = loadUser();
  expect(aliased).toBeDefined();
  const name = aliased!.name;
  expect(name).toBe("Alice");

  const asyncUser = loadUser() as (User & { load(): Promise<string> }) | undefined;
  expect(asyncUser).toBeDefined();
  expect(await asyncUser?.load()).toBe("A");

  const containUser = loadUser();
  expect(containUser).toBeDefined();
  expect(containUser?.name).toContain("A");

  const objMatch = loadUser();
  expect(objMatch).toBeDefined();
  expect(objMatch?.profile).toEqual({ email: "a@example.com" });

  const arrMatch = loadUser();
  expect(arrMatch).toBeDefined();
  expect(arrMatch?.tags).toEqual(["admin"]);

  // Counterexamples (allowed)
  const withOther = loadUser();
  expect(withOther).toBeDefined();
  expect(unrelatedName).toBe("A");
  const alone = loadUser();
  expect(alone).toBeDefined();

  const optionalUndef = loadUser();
  expect(optionalUndef).toBeDefined();
  expect(optionalUndef?.nickname).toBeUndefined();

  const optionalEqUndef = loadUser();
  expect(optionalEqUndef).toBeDefined();
  expect(optionalEqUndef?.nickname).toBe(undefined);

  const optionalNegated = loadUser();
  expect(optionalNegated).toBeDefined();
  expect(optionalNegated?.nickname).not.toBe("Bob");

  const optionalFalsy = loadUser();
  expect(optionalFalsy).toBeDefined();
  expect(optionalFalsy?.nickname).toBeFalsy();

  const first = loadUser();
  const second = loadUser();
  expect(first).toBeDefined();
  expect(second!.name).toBe("Bob");

  let reassigned = loadUser();
  expect(reassigned).toBeDefined();
  reassigned = loadUser();
  expect(reassigned!.name).toBe("Bob");

  const undefAlias = undefined;
  const withAliasUndef = loadUser();
  expect(withAliasUndef).toBeDefined();
  expect(withAliasUndef?.nickname).toBe(undefAlias);

  let preAlias = loadUser();
  const captured = preAlias?.name;
  preAlias = undefined;
  expect(preAlias).toBeDefined();
  expect(captured).toBe("Alice");

  const state = loadResult();
  expect(state.item).toBeDefined();
  state.item = { status: "new" };
  expect(state.item!.status).toBe("new");

  const delState = loadResult();
  expect(delState.item).toBeDefined();
  delete delState.item;
  expect(delState.item!.status).toBe("new");

  const early = loadUser();
  expect(early).toBeDefined();
  if (!early) return;
  expect(early!.name).toBe("Alice");

  const typeCheck = loadUser();
  expect(typeCheck).toBeDefined();
  expect(typeCheck?.nickname).toBeTypeOf("undefined");

  const nonNullOpt = loadUser();
  expect(nonNullOpt).toBeDefined();
  expect(nonNullOpt!?.nickname).toBeUndefined();

  let deferredUser: User | undefined;
  function initialize() {
    deferredUser = loadUser();
  }
  expect(deferredUser).toBeDefined();
  initialize();
  expect(deferredUser!.name).toBe("Ada");
}

export function definedThenDerefWithParam(expected: string | undefined) {
  const maybe = loadUser();
  expect(maybe).toBeDefined();
  expect(maybe?.name).toBe(expected);
}

export async function definedThenAwaitIdentifier(state: { user?: { name: string } }, ready: Promise<void>) {
  expect(state.user).toBeDefined();
  await ready;
  expect(state.user!.name).toBe("Ada");
}
