import { read, sql } from "@example/db";

const COLS = sql`
d.id,
d.item_id,
d.group_id,
d.party_id,
d.status,
d.c06,
d.c07,
d.c08,
d.c09,
d.c10,
d.c11,
d.c12,
d.c13,
d.c14,
d.c15,
d.c16,
d.c17,
d.c18,
d.c19,
d.c20,
d.c21,
d.c22,
d.c23,
d.c24,
d.c25,
d.c26,
d.c27,
d.c28,
d.c29,
d.c30,
d.c31,
d.c32,
d.c33,
d.c34,
d.c35,
d.c36,
d.c37,
d.c38,
d.c39,
d.c40,
d.c41,
d.c42,
d.c43,
d.c44,
d.c45,
d.c46,
d.c47,
d.c48,
d.c49,
d.c50
`;

const JOINS = sql`
LEFT JOIN dispute_parties p ON p.id = d.party_id
JOIN items i ON i.id = d.item_id
LEFT JOIN groups g ON g.id = d.group_id AND g.active
LEFT JOIN LATERAL (
  SELECT s.id
  FROM settlements s
  WHERE s.dispute_id = d.id
  ORDER BY s.id DESC
  LIMIT 1
) latest ON true
`;

export function capped(id: string) {
  return read(
    sql`SELECT `
      .append(COLS)
      .append(sql` FROM disputes d `)
      .append(JOINS)
      .append(sql` WHERE d.id = ${id} LIMIT 1`),
  );
}

export function conditional(id: string, n: number, extra: boolean) {
  const q = sql`SELECT d.id `;
  q.append(sql`FROM disputes d `);
  q.append(sql`JOIN items i ON i.id = d.item_id `);
  if (extra) {
    q.append(sql`WHERE d.id = ${id} `);
  } else {
    q.append(sql`WHERE d.item_id = ${id} `);
  }
  q.append(sql`LIMIT ${n}`);
  return read(q);
}

export function fanout(id: string) {
  return read(
    sql`SELECT `
      .append(COLS)
      .append(sql` FROM disputes d `)
      .append(JOINS)
      .append(sql` WHERE d.item_id = ${id} ORDER BY d.id DESC`),
  );
}
