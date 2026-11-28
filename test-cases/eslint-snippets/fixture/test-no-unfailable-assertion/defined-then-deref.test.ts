import { expect } from "vitest";

interface User {
  name: string;
  nickname?: string;
  profile: { email: string };
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
}
