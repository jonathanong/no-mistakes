import { sql } from "@example/sql";
import { external } from "./external.ts";

external("ordinary");

function nullableSink(value) {
  const nullableCallerSlot = sql`records WHERE value = ${value}`;
}
nullableSink(null);
nullableSink(undefined);

function awaitedSink(value) {
  const awaitedCallerSlot = sql`records WHERE value = ${value}`;
}
awaitedSink(await "ordinary");

function nullishSink(value) {
  const nullishCallerSlot = sql`records WHERE value = ${value}`;
}
nullishSink(null ?? undefined);

function memberSink(value) {
  const memberCallerSlot = sql`records WHERE value = ${value}`;
}
memberSink({ value: "ordinary" }.value);

function arrayMemberSink(value) {
  const arrayMemberCallerSlot = sql`records WHERE value = ${value}`;
}
arrayMemberSink(["ordinary"][0]);

function containerSink(value) {
  const containerCallerSlot = sql`records WHERE value = ${value}`;
}
containerSink({ value: "ordinary" });

export async function directSlots() {
  const undefinedBindSlot = sql`records WHERE value = ${undefined}`;
  const awaitedBindSlot = sql`records WHERE value = ${await null}`;
  const nullishBindSlot = sql`records WHERE value = ${null ?? undefined}`;
}

function nestedDefault({ group: [item = null] } = { group: [null] }) {
  const defaultProjectionSlot = sql`records WHERE value = ${item}`;
}
nestedDefault();
