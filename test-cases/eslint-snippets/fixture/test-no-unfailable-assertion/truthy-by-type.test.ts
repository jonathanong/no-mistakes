import { expect } from "vitest";

interface User {
  id: string;
}
type BrandedId = string & { readonly __brand: "Id" };

declare function getUser(): User;
declare function getHandler(): () => void;
declare function getCtor(): new () => User;
declare function getRecord(): Record<string, number>;
declare function getObject(): object;
declare function getSymbol(): symbol;
declare function getTruthyUnion(): User | (() => void);
declare function getTruthyLiteralUnion(): "ok" | "done";
declare function getOne(): 1;
declare function getBigOne(): 1n;
declare function getTrue(): true;
declare function getObjIntersection(): { a: number } & { b: string };

declare function getString(): string;
declare function getNumber(): number;
declare function getBoolean(): boolean;
declare function getBigInt(): bigint;
declare function getNullableUser(): User | null;
declare function getOptionalUser(): User | undefined;
declare function getAny(): any;
declare function getUnknown(): unknown;
declare function getEmptyObj(): {};
declare function getBoxedObject(): Object;
declare function getBoxedString(): String;
declare function getBrandedId(): BrandedId;
declare function getMaybeEmptyString(): "" | "ok";
declare function getMaybeZero(): 0 | 1;
declare function getMaybeZeroBigInt(): 0n | 1n;
declare function getFalse(): false;
declare const users: User[];
declare const userMap: Record<string, User>;
declare const mapKey: string;

// Positive cases (truthy-by-type)
expect(getUser()).toBeTruthy();
expect(getUser()).not.toBeFalsy();
expect(getHandler()).toBeTruthy();
expect(getCtor()).toBeTruthy();
expect([1, 2]).toBeTruthy();
expect(["a", 1] as const).toBeTruthy();
expect({ id: "1" }).toBeTruthy();
expect(() => {}).toBeTruthy();
expect(getRecord()).toBeTruthy();
expect(getObject()).toBeTruthy();
expect(getSymbol()).toBeTruthy();
expect(getTruthyUnion()).toBeTruthy();
expect(getTruthyLiteralUnion()).toBeTruthy();
expect(getOne()).toBeTruthy();
expect(getBigOne()).toBeTruthy();
expect(getTrue()).toBeTruthy();
expect(getObjIntersection()).toBeTruthy();

export function constrainedTruthy<T extends User>(value: T) {
  expect(value).toBeTruthy();
}

// Counterexamples (allowed)
expect(getString()).toBeTruthy();
expect(getNumber()).toBeTruthy();
expect(getBoolean()).toBeTruthy();
expect(getBigInt()).toBeTruthy();
expect(getNullableUser()).toBeTruthy();
expect(getOptionalUser()).toBeTruthy();
expect(getAny()).toBeTruthy();
expect(getUnknown()).toBeTruthy();
expect(getEmptyObj()).toBeTruthy();
expect(getBoxedObject()).toBeTruthy();
expect(getBoxedString()).toBeTruthy();
expect(getBrandedId()).toBeTruthy();
expect(getMaybeEmptyString()).toBeTruthy();
expect(getMaybeZero()).toBeTruthy();
expect(getMaybeZeroBigInt()).toBeTruthy();
expect(getFalse()).toBeTruthy();
expect(users[0]).toBeTruthy();
expect(userMap[mapKey]).toBeTruthy();

export function unconstrainedTruthy<T>(value: T) {
  expect(value).toBeTruthy();
}
