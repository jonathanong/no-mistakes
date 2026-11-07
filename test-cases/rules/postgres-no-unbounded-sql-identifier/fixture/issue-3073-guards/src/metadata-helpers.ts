import { query } from "@example/db";

const TABLES = [
  { name: "user", table: "users" },
  { name: "post", table: "posts" },
] as const;

function getTableOrThrow(name: string) {
  const selected = TABLES.find((entry) => entry.name === name);
  if (!selected) throw new Error("Unknown table");
  return selected.table;
}

export async function literalLookupHelper() {
  await query(`SELECT * FROM ${getTableOrThrow("user")}`);
}

export async function assignedLookupHelper(name: string) {
  const table = getTableOrThrow(name);
  await query(`SELECT * FROM ${table}`);
}

export async function dynamicLookupHelper(name: string) {
  await query(`SELECT * FROM ${getTableOrThrow(name)}`);
}

function unchecked(name: string) {
  const selected = TABLES.find((entry) => entry.name === name);
  return selected.table;
}

export async function uncheckedHelper(name: string) {
  const table = unchecked(name);
  await query(`SELECT * FROM ${table}`);
}

function withFallback(name: string) {
  const selected = TABLES.find((entry) => entry.name === name);
  if (!selected) return name;
  return selected.table;
}

export async function openFallbackHelper(name: string) {
  const table = withFallback(name);
  await query(`SELECT * FROM ${table}`);
}

const OPEN_TABLES = [{ name: "user", table: process.env.TABLE }];
function open(name: string) {
  const selected = OPEN_TABLES.find((entry) => entry.name === name);
  if (!selected) throw new Error("Unknown table");
  return selected.table;
}

export async function openLookupHelper(name: string) {
  const table = open(name);
  await query(`SELECT * FROM ${table}`);
}

async function asyncLookup(name: string) {
  const selected = TABLES.find((entry) => entry.name === name);
  if (!selected) throw new Error("Unknown table");
  return selected.table;
}

export async function asyncLookupHelper(name: string) {
  const table = asyncLookup(name);
  await query(`SELECT * FROM ${table}`);
}

const OBJECT_TABLES = [{ name: "user", table: "users" }];
function objectLookup(name: string) {
  const selected = OBJECT_TABLES.find((entry) => entry.name === name);
  if (!selected) throw new Error("Unknown table");
  return selected;
}

export async function objectLookupHelper(name: string) {
  const selected = objectLookup(name);
  await query(`SELECT * FROM ${selected.table}`);
}

const TAINTED_TABLES = [{ name: "user", table: "users" }];
function mutateLookup(name: string, table: string) {
  const selected = TAINTED_TABLES.find((entry) => entry.name === name);
  if (!selected) throw new Error("Unknown table");
  selected.table = table;
}

function taintedLookup(name: string) {
  const selected = TAINTED_TABLES.find((entry) => entry.name === name);
  if (!selected) throw new Error("Unknown table");
  return selected.table;
}

export async function taintedLookupHelper(name: string) {
  const table = taintedLookup(name);
  await query(`SELECT * FROM ${table}`);
}

const RAW_TABLES = [{ name: "user", table: "users" }];
RAW_TABLES.find((entry) => entry.name === "user")!.table = process.env.TABLE;
function rawLookup(name: string) {
  const selected = RAW_TABLES.find((entry) => entry.name === name);
  if (!selected) throw new Error("Unknown table");
  return selected.table;
}

export async function rawLookupHelper(name: string) {
  const table = rawLookup(name);
  await query(`SELECT * FROM ${table}`);
}
