import { run, transform061 } from "../src/helper.mjs";

export async function case61_00(mode: number) {
  const value = transform061("caller-61-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case61_01(mode: number) {
  const value = transform061("caller-61-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case61_02(mode: number) {
  const value = transform061("caller-61-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case61_03(mode: number) {
  const value = transform061("caller-61-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case61_04(mode: number) {
  const value = transform061("caller-61-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case61_05(mode: number) {
  const value = transform061("caller-61-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case61_06(mode: number) {
  const value = transform061("caller-61-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case61_07(mode: number) {
  const value = transform061("caller-61-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case61_08(mode: number) {
  const value = transform061("caller-61-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case61_09(mode: number) {
  const value = transform061("caller-61-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case61_10(mode: number) {
  const value = transform061("caller-61-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case61_11(mode: number) {
  const value = transform061("caller-61-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case61_12(mode: number) {
  const value = transform061("caller-61-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case61_13(mode: number) {
  const value = transform061("caller-61-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case61_14(mode: number) {
  const value = transform061("caller-61-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case61_15(mode: number) {
  const value = transform061("caller-61-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}
