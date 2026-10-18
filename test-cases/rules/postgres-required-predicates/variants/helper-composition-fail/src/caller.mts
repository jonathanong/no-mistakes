import sql from "sql-template-strings";
import { read } from "@example/db";
import {
  ping,
  overParameter,
  whileLoop,
  manyAlternatives,
  untypedBuilder,
  undeclaredCallee,
  throughParameter,
  rawIdentifier,
  shadowedTag,
  localTag,
  handsOverBuilder,
} from "./helpers.mts";

// Every read below stays unanalyzable: each helper crosses a boundary the
// recovery refuses to guess across.

export async function cycle() {
  return read(ping());
}

export async function loopOverParameter(tables: string[]) {
  return read(overParameter(tables));
}

export async function loopWithoutBound() {
  return read(whileLoop());
}

export async function tooManyAlternatives(options: Record<string, boolean>) {
  return read(manyAlternatives(options));
}

export async function untypedParameter() {
  const query = sql`SELECT 1 FROM roots r WHERE r.id = 1`;
  untypedBuilder(query);
  return read(query);
}

export async function unknownCallee() {
  return read(undeclaredCallee());
}

export async function calledThroughParameter() {
  return read(throughParameter(() => sql`SELECT 1`));
}

export async function identifierInRawText(column: string) {
  return read(rawIdentifier(column));
}

export async function reboundTag() {
  return read(shadowedTag(sql));
}

export async function reboundLocalTag() {
  return read(localTag(sql));
}

export async function unresolvedCalleeGetsBuilder() {
  const query = sql`SELECT 1 FROM roots r WHERE r.id = 1`;
  handsOverBuilder(query);
  return read(query);
}
