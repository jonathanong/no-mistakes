import { query } from "@bench/db";

export async function case21_00(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case21_01(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case21_02(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case21_03(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case21_04(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case21_05(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case21_06(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case21_07(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case21_08(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case21_09(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case21_10(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case21_11(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case21_12(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case21_13(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case21_14(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case21_15(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}
