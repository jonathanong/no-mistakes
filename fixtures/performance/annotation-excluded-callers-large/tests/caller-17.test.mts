import { run, transform017 } from "../src/helper.mjs";

export async function case17_00(mode: number) {
  const value = transform017("caller-17-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case17_01(mode: number) {
  const value = transform017("caller-17-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case17_02(mode: number) {
  const value = transform017("caller-17-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case17_03(mode: number) {
  const value = transform017("caller-17-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case17_04(mode: number) {
  const value = transform017("caller-17-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case17_05(mode: number) {
  const value = transform017("caller-17-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case17_06(mode: number) {
  const value = transform017("caller-17-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case17_07(mode: number) {
  const value = transform017("caller-17-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case17_08(mode: number) {
  const value = transform017("caller-17-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case17_09(mode: number) {
  const value = transform017("caller-17-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case17_10(mode: number) {
  const value = transform017("caller-17-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case17_11(mode: number) {
  const value = transform017("caller-17-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case17_12(mode: number) {
  const value = transform017("caller-17-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case17_13(mode: number) {
  const value = transform017("caller-17-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case17_14(mode: number) {
  const value = transform017("caller-17-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case17_15(mode: number) {
  const value = transform017("caller-17-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}
