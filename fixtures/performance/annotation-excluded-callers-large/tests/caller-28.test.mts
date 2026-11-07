import { run, transform028 } from "../src/helper.mjs";

export async function case28_00(mode: number) {
  const value = transform028("caller-28-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case28_01(mode: number) {
  const value = transform028("caller-28-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case28_02(mode: number) {
  const value = transform028("caller-28-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case28_03(mode: number) {
  const value = transform028("caller-28-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case28_04(mode: number) {
  const value = transform028("caller-28-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case28_05(mode: number) {
  const value = transform028("caller-28-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case28_06(mode: number) {
  const value = transform028("caller-28-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case28_07(mode: number) {
  const value = transform028("caller-28-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case28_08(mode: number) {
  const value = transform028("caller-28-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case28_09(mode: number) {
  const value = transform028("caller-28-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case28_10(mode: number) {
  const value = transform028("caller-28-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case28_11(mode: number) {
  const value = transform028("caller-28-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case28_12(mode: number) {
  const value = transform028("caller-28-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case28_13(mode: number) {
  const value = transform028("caller-28-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case28_14(mode: number) {
  const value = transform028("caller-28-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case28_15(mode: number) {
  const value = transform028("caller-28-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}
