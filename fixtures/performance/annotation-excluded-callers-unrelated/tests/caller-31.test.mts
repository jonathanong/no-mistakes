import { query } from "@bench/db";

export async function case31_00(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case31_01(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case31_02(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case31_03(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case31_04(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case31_05(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case31_06(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case31_07(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case31_08(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case31_09(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case31_10(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case31_11(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case31_12(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case31_13(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case31_14(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case31_15(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}
