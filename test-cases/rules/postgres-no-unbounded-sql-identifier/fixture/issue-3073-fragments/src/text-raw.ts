import { query } from "@example/db";
import { sql } from "@example/sql";
import legacy from "sql-template-strings";
import type { UserId } from "@example/types";

declare function makeRaw(value: string): unknown;

export function literalAndScalarValues(value: string, config: { value: string }) {
  const literals = sql`records WHERE first = ${"fixed"} AND second = ${2}`;
  query(`SELECT id FROM ${literals.text}`);
  const scalarAlias = value;
  const aliasedScalar = sql`records WHERE value = ${scalarAlias}`;
  query(`SELECT id FROM ${aliasedScalar.text}`);
  const parameterMember = sql`records WHERE value = ${config.value}`;
  query(`SELECT id FROM ${parameterMember.text}`);
  const object = { value };
  const localMember = sql`records WHERE value = ${object.value}`;
  query(`SELECT id FROM ${localMember.text}`);
  const legacyRawValue = legacy`records WHERE value = ${makeRaw?.(value)}`;
  query(`SELECT id FROM ${legacyRawValue.text}`);
  query(`SELECT id FROM ${sql`records WHERE value = ${process.env.BOUND_VALUE}`.text}`);
}

export function rawValues(value: string, flag: boolean) {
  const rawSlot = sql`records ${sql.raw(value)}`;
  query(`SELECT id FROM ${rawSlot.text}`);
  const rawArgument = sql.raw(value);
  const rawAlias = rawArgument;
  const rawAliasSlot = sql`records ${rawAlias}`;
  query(`SELECT id FROM ${rawAliasSlot.text}`);
  const rawFactory = sql.raw;
  const rawFactorySlot = sql`records ${rawFactory(value)}`;
  query(`SELECT id FROM ${rawFactorySlot.text}`);
  const opaqueSlot = sql`records ${makeRaw(value)}`;
  query(`SELECT id FROM ${opaqueSlot.text}`);
  const optionalSlot = sql`records ${makeRaw?.(value)}`;
  query(`SELECT id FROM ${optionalSlot.text}`);
  const optionalRaw = makeRaw?.(value);
  const optionalAliasSlot = sql`records ${optionalRaw}`;
  query(`SELECT id FROM ${optionalAliasSlot.text}`);
  const rawObject = { text: value, values: [] };
  const objectSlot = sql`records ${rawObject}`;
  query(`SELECT id FROM ${objectSlot.text}`);
  const objectWithRaw = { value: makeRaw(value) };
  const objectMemberSlot = sql`records ${objectWithRaw.value}`;
  query(`SELECT id FROM ${objectMemberSlot.text}`);
  const mixedArgument = flag ? sql.raw(value) : process.env.MIXED_VALUE;
  const mixedSlot = sql`records ${mixedArgument}`;
  query(`SELECT id FROM ${mixedSlot.text}`);
  query(`SELECT id FROM ${sql`records ${sql.raw(process.env.INLINE_RAW)}`.text}`);
  query(`SELECT id FROM ${sql`records ${rawAlias} ${process.env.ALIAS_TABLE}`.text}`);
  query(`SELECT id FROM ${sql`records ${makeRaw?.(value)} ${process.env.OPTIONAL_TABLE}`.text}`);
  query(sql`SELECT id FROM ${process.env.DIRECT_TABLE} WHERE id = ${sql.raw(value)}`);
}

export function aliasDiamond(value: string) {
  const a0 = value;
  const a1 = `${a0}${a0}`;
  const a2 = `${a1}${a1}`;
  const a3 = `${a2}${a2}`;
  const a4 = `${a3}${a3}`;
  const a5 = `${a4}${a4}`;
  const a6 = `${a5}${a5}`;
  const a7 = `${a6}${a6}`;
  const a8 = `${a7}${a7}`;
  const a9 = `${a8}${a8}`;
  const a10 = `${a9}${a9}`;
  const diamond = sql`records WHERE value = ${a10}`;
  query(`SELECT id FROM ${diamond.text}`);
}

export function optionalDefault(value: string = makeRaw?.("raw") as string) {
  const optionalDefaultSlot = sql`records WHERE value = ${value}`;
  query(`SELECT id FROM ${optionalDefaultSlot.text}`);
  query(`SELECT id FROM ${sql`records WHERE value = ${value}`.text}`);
}

export function rawDefault(value: any = sql.raw("raw")) {
  const rawDefaultSlot = sql`records WHERE value = ${value}`;
  query(`SELECT id FROM ${rawDefaultSlot.text}`);
}

export function aliasDefault(original: any = makeRaw?.("raw"), value: any = original) {
  const alias = value;
  const aliasDefaultSlot = sql`records WHERE value = ${alias}`;
  query(`SELECT id FROM ${aliasDefaultSlot.text}`);
}

export function capturedDefault(value: any = makeRaw?.("raw")) {
  const captured = value;
  return function inner(value: string) {
    const capturedDefaultSlot = sql`records WHERE value = ${captured}`;
    query(`SELECT id FROM ${capturedDefaultSlot.text}`);
  };
}

export function literalDefault(value: string = "fixed") {
  const literalDefaultSlot = sql`records WHERE value = ${value}`;
  query(`SELECT id FROM ${literalDefaultSlot.text}`);
}

export function untyped(value) {
  const untypedSlot = sql`records WHERE value = ${value}`;
  query(`SELECT id FROM ${untypedSlot.text}`);
}

export function destructured({ value }) {
  const destructuredSlot = sql`records WHERE value = ${value}`;
  query(`SELECT id FROM ${destructuredSlot.text}`);
}

export function nominal(value: UserId) {
  const nominalSlot = sql`records WHERE value = ${value}`;
  query(`SELECT id FROM ${nominalSlot.text}`);
}

export function destructuredLiteralDefault({ value = "fixed" }) {
  const destructuredLiteralSlot = sql`records WHERE value = ${value}`;
  query(`SELECT id FROM ${destructuredLiteralSlot.text}`);
}

export function destructuredRawDefault({ value = sql.raw("raw") }) {
  const destructuredRawSlot = sql`records WHERE value = ${value}`;
  query(`SELECT id FROM ${destructuredRawSlot.text}`);
}

export function destructuredOptionalDefault({ value = makeRaw?.("raw") }) {
  const destructuredOptionalSlot = sql`records WHERE value = ${value}`;
  query(`SELECT id FROM ${destructuredOptionalSlot.text}`);
}
