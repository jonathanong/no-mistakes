import { run, transform048 } from "../src/helper.mjs";

export async function case48_00(mode: number) {
  const value = transform048("caller-48-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case48_01(mode: number) {
  const value = transform048("caller-48-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case48_02(mode: number) {
  const value = transform048("caller-48-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case48_03(mode: number) {
  const value = transform048("caller-48-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case48_04(mode: number) {
  const value = transform048("caller-48-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case48_05(mode: number) {
  const value = transform048("caller-48-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case48_06(mode: number) {
  const value = transform048("caller-48-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case48_07(mode: number) {
  const value = transform048("caller-48-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case48_08(mode: number) {
  const value = transform048("caller-48-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case48_09(mode: number) {
  const value = transform048("caller-48-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case48_10(mode: number) {
  const value = transform048("caller-48-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case48_11(mode: number) {
  const value = transform048("caller-48-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case48_12(mode: number) {
  const value = transform048("caller-48-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case48_13(mode: number) {
  const value = transform048("caller-48-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case48_14(mode: number) {
  const value = transform048("caller-48-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case48_15(mode: number) {
  const value = transform048("caller-48-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}
