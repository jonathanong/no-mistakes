import { beginTransaction } from "@example/db";

export async function keyUpdate() {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM jobs WHERE id = ':x' FOR KEY SHARE`);
  await query(`UPDATE jobs SET id = ':y' WHERE id = ':x'`);
}

export async function nonKeyUpdate() {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM jobs WHERE id = ':x' FOR KEY SHARE`);
  await query(`UPDATE jobs SET state = 'done' WHERE id = ':x'`);
}

export async function differentRow() {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM jobs WHERE id = ':x' FOR KEY SHARE`);
  await query(`UPDATE jobs SET id = ':z' WHERE id = ':y'`);
}

export async function keyAssignmentInBatch() {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM jobs WHERE id = ':x' FOR KEY SHARE;
    UPDATE jobs SET (id, state) = (':y', 'done') WHERE id = ':x'`);
}

export async function keyUnchanged() {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM jobs WHERE id = ':x' FOR KEY SHARE`);
  await query(`UPDATE jobs SET id = id WHERE id = ':x'`);
}

export async function relationWideKeyUpdate() {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM jobs WHERE id = ':x' FOR KEY SHARE`);
  await query(`UPDATE jobs SET id = id || '-archived'`);
}

export async function literalMatchesWithinBatch() {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM jobs WHERE id = ':x' FOR SHARE;
    DELETE FROM jobs WHERE id = ':x'`);
}
