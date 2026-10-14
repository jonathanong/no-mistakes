import { read, sql } from '@example/db';

function fill(parts: string[], flag: boolean) {
  if (flag) parts.push('a');
  else parts.push('a', 'b');
}

export function conditionalForEach(flag: boolean) {
  const query = sql`SELECT 1`;
  const parts: string[] = [];
  fill(parts, flag);
  // Joined shape metadata must not hide the possible second callback.
  parts.forEach((_, index) => {
    if (index === 1) query.append(sql`; DELETE FROM records`);
  });
  read(query);
}

export function conditionalMap(flag: boolean) {
  const query = sql`SELECT 1`;
  const parts: string[] = [];
  fill(parts, flag);
  parts.map((_, index) => {
    if (index === 1) query.append(sql`; DELETE FROM records`);
    return 'part';
  });
  read(query);
}

export function denseForEach() {
  const query = sql`SELECT 1`;
  ['a', 'b'].forEach((_, index) => {
    if (index === 1) query.append(sql`; DELETE FROM records`);
  });
  read(query);
}

export function denseMap() {
  const query = sql`SELECT 1`;
  ['a', 'b'].map((_, index) => {
    if (index === 1) query.append(sql`; DELETE FROM records`);
    return 'part';
  });
  read(query);
}

export function conditionalPush(flag: boolean) {
  const parts: string[] = [];
  fill(parts, flag);
  const length = parts.push('c');
  read(length === 2 ? 'SELECT 1' : 'DELETE FROM records');
}

export function conditionalMapResult(flag: boolean) {
  const parts: string[] = [];
  fill(parts, flag);
  read(parts.map((_, index) => index === 0 ? 'SELECT 1' : '; DELETE FROM records').join(''));
}

export function conditionalOverwrite(flag: boolean) {
  const query = sql`SELECT 1`;
  const parts: string[] = [];
  fill(parts, flag);
  // Assigning one slot cannot establish the array's divergent runtime length.
  parts[0] = 'c';
  parts.forEach((_, index) => {
    if (index === 1) query.append(sql`; DELETE FROM records`);
  });
  read(query);
}
