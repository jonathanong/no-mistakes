import { query } from "@bench/db";

export async function case23_00(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case23_01(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case23_02(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case23_03(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case23_04(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case23_05(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case23_06(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case23_07(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case23_08(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case23_09(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case23_10(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case23_11(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case23_12(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case23_13(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case23_14(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}

export async function case23_15(mode: number) {
  const value = mode > 0 ? "active" : "idle";
  const result = await query("SELECT id FROM items");
  const records = mode > 1 ? [result, value, String(mode)] : [String(mode), value, result];
  return records.map((record) => ({ record, mode }));
}
