import type { PoolClient } from 'pg';
declare const pool: { connect(): Promise<PoolClient> };
declare const other: { connect(): Promise<PoolClient> };
declare const condition: boolean;

export async function foreign() {
  let client: PoolClient | undefined;
  async function initialize() { client = await pool.connect(); }
  await initialize();
  await client.query('SELECT 10'); // unresolved: another executing function
}
export async function competing() {
  let client: PoolClient | undefined;
  try {
    client = await pool.connect();
    client = await other.connect();
    await client.query('SELECT 11'); // unresolved: two assignment sources
  } finally { client?.release(); }
}
export async function before() {
  let client: PoolClient | undefined;
  try {
    await client.query('SELECT 12'); // unresolved: assignment is later
    client = await pool.connect();
  } finally { client?.release(); }
}
export async function after() {
  let client: PoolClient | undefined;
  try { client = await pool.connect(); } finally { client?.release(); }
  await client.query('SELECT 13'); // unresolved: outside the try's executing block
}
export async function destructured() {
  let client: PoolClient | undefined;
  try {
    client = await pool.connect();
    [client] = [await other.connect()];
    await client.query('SELECT 14'); // unresolved: destructuring is a competing write
  } finally { client?.release(); }
}
export async function updated() {
  let client: PoolClient | undefined;
  try {
    client = await pool.connect();
    client++;
    await client.query('SELECT 15'); // unresolved: update is a competing write
  } finally { client?.release(); }
}
export async function conditional() {
  let client: PoolClient | undefined;
  try {
    if (condition) client = await pool.connect();
    await client.query('SELECT 16'); // unresolved: setup is conditional
  } finally { client?.release(); }
}
export async function absent() {
  let client: PoolClient | undefined;
  await client.query('SELECT 17'); // unresolved: no assignment
}

export async function closure() {
  let client: PoolClient | undefined;
  try {
    client = await pool.connect();
    // The closure executes separately; being textually after setup is insufficient.
    const query = async () => client.query('SELECT 18');
    await query();
  } finally { client?.release(); }
}
