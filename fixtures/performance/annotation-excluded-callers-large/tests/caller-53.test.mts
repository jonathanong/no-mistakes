import { run, transform053 } from "../src/helper.mjs";

export async function case53_00(mode: number) {
  const value = transform053("caller-53-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case53_01(mode: number) {
  const value = transform053("caller-53-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case53_02(mode: number) {
  const value = transform053("caller-53-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case53_03(mode: number) {
  const value = transform053("caller-53-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case53_04(mode: number) {
  const value = transform053("caller-53-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case53_05(mode: number) {
  const value = transform053("caller-53-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case53_06(mode: number) {
  const value = transform053("caller-53-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case53_07(mode: number) {
  const value = transform053("caller-53-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case53_08(mode: number) {
  const value = transform053("caller-53-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case53_09(mode: number) {
  const value = transform053("caller-53-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case53_10(mode: number) {
  const value = transform053("caller-53-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case53_11(mode: number) {
  const value = transform053("caller-53-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case53_12(mode: number) {
  const value = transform053("caller-53-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case53_13(mode: number) {
  const value = transform053("caller-53-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case53_14(mode: number) {
  const value = transform053("caller-53-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}

export async function case53_15(mode: number) {
  const value = transform053("caller-53-value", mode > 0);
  const first = await run("SELECT id FROM items", mode);
  const second = await run(`SELECT name FROM items WHERE name = '${value}'`, mode + 1);
  const records = mode > 0 ? [first, second] : [second, first];
  return records.map((record) => ({ record, mode, value }));
}
