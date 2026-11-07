import { read } from "@example/db";
import sql from "sql-template-strings";

export async function parameterFrontier(parentIds: string[], rowLimits: number[]) {
  const { rows } = await read(sql`/* parameterFrontier */
    SELECT p.id
    FROM unnest(${parentIds}::uuid[], ${rowLimits}::int[])
      AS frontier(parent_id, row_limit)
    CROSS JOIN LATERAL (
      SELECT child.id
      FROM posts child
      WHERE child.parent_id = frontier.parent_id
      ORDER BY child.id
      LIMIT frontier.row_limit
    ) p
  `);
  return rows;
}

export async function parameterFrontierFetch(parentIds: string[], rowLimits: number[]) {
  const { rows } = await read(sql`/* parameterFrontierFetch */
    SELECT p.id
    FROM unnest(${parentIds}::uuid[], ${rowLimits}::int[])
      WITH ORDINALITY AS frontier(parent_id, row_limit, position)
    CROSS JOIN LATERAL (
      SELECT child.id
      FROM posts child
      WHERE child.parent_id = frontier.parent_id
      ORDER BY child.id
      FETCH FIRST (frontier.row_limit) ROWS ONLY
    ) p
  `);
  return rows;
}

export async function parameterFrontierArrayZip(
  parentIds: string[],
  rowLimits: number[],
  positions: number[],
) {
  const { rows } = await read(sql`/* parameterFrontierArrayZip */
    SELECT p.id
    FROM unnest(${parentIds}::uuid[], ${rowLimits}::int[], ${positions}::int[])
      AS frontier(parent_id, row_limit, position)
    CROSS JOIN LATERAL (
      SELECT child.id
      FROM posts child
      WHERE child.parent_id = frontier.parent_id
      ORDER BY child.id
      LIMIT frontier.row_limit
    ) p
  `);
  return rows;
}

export async function relationFrontier(parentId: string) {
  const { rows } = await read(sql`/* relationFrontier */
    SELECT p.id
    FROM limits frontier
    CROSS JOIN LATERAL (
      SELECT child.id
      FROM posts child
      WHERE child.parent_id = ${parentId}
      ORDER BY child.id
      LIMIT frontier.row_limit
    ) p
  `);
  return rows;
}

export async function tableArrayFrontier(parentId: string) {
  const { rows } = await read(sql`/* tableArrayFrontier */
    SELECT p.id
    FROM (
      SELECT parent_ids, row_limits
      FROM frontiers
      WHERE id = ${parentId}
      LIMIT 1
    ) source
    CROSS JOIN LATERAL unnest(source.parent_ids, source.row_limits)
      AS frontier(parent_id, row_limit)
    CROSS JOIN LATERAL (
      SELECT child.id
      FROM posts child
      WHERE child.parent_id = frontier.parent_id
      ORDER BY child.id
      LIMIT frontier.row_limit
    ) p
  `);
  return rows;
}

export async function quotedParameterFrontier(parentIds: string[], rowLimits: number[]) {
  const { rows } = await read(sql`/* quotedParameterFrontier */
    SELECT p.id
    FROM LATERAL unnest(${parentIds}::uuid[], ${rowLimits}::int[])
      AS "frontier"("parentId", "rowLimit")
    CROSS JOIN LATERAL (
      SELECT child.id
      FROM posts child
      WHERE child.parent_id = "frontier"."parentId"
      ORDER BY child.id
      LIMIT "frontier"."rowLimit"
    ) p
  `);
  return rows;
}

export async function mixedParameterAndTableArrays(parentId: string, rowLimits: number[]) {
  const { rows } = await read(sql`/* mixedParameterAndTableArrays */
    SELECT p.id
    FROM (
      SELECT parent_ids
      FROM frontiers
      WHERE id = ${parentId}
      LIMIT 1
    ) source
    CROSS JOIN LATERAL unnest(source.parent_ids, ${rowLimits}::int[])
      AS frontier(parent_id, row_limit)
    CROSS JOIN LATERAL (
      SELECT child.id
      FROM posts child
      WHERE child.parent_id = frontier.parent_id
      ORDER BY child.id
      LIMIT frontier.row_limit
    ) p
  `);
  return rows;
}

export async function nonLateralDerivedFrontier(parentIds: string[], rowLimits: number[]) {
  const { rows } = await read(sql`/* nonLateralDerivedFrontier */
    SELECT p.id
    FROM (
      SELECT frontier.parent_id, frontier.row_limit
      FROM unnest(${parentIds}::uuid[], ${rowLimits}::int[])
        AS frontier(parent_id, row_limit)
    ) frontier
    CROSS JOIN LATERAL (
      SELECT child.id
      FROM posts child
      WHERE child.parent_id = frontier.parent_id
      ORDER BY child.id
      LIMIT frontier.row_limit
    ) p
  `);
  return rows;
}

export async function nullableParameterFrontier(
  parentIds: string[],
  rowLimits: number[],
  enabled: boolean,
) {
  const { rows } = await read(sql`/* nullableParameterFrontier */
    SELECT p.id
    FROM (SELECT ${enabled}::boolean AS enabled) source
    LEFT JOIN LATERAL unnest(${parentIds}::uuid[], ${rowLimits}::int[])
      AS frontier(parent_id, row_limit) ON source.enabled
    CROSS JOIN LATERAL (
      SELECT child.id
      FROM posts child
      WHERE child.parent_id = frontier.parent_id
      ORDER BY child.id
      LIMIT frontier.row_limit
    ) p
  `);
  return rows;
}

export async function shadowedFrontier(parentIds: string[], rowLimits: number[]) {
  const { rows } = await read(sql`/* shadowedFrontier */
    SELECT p.id
    FROM unnest(${parentIds}::uuid[], ${rowLimits}::int[])
      AS frontier(parent_id, row_limit)
    CROSS JOIN LATERAL (
      SELECT child.id
      FROM posts frontier
      JOIN posts child ON child.parent_id = frontier.parent_id
      ORDER BY child.id
      LIMIT frontier.row_limit
    ) p
  `);
  return rows;
}

export async function statementTextAndValues(parentIds: string[], rowLimits: number[]) {
  const statement = sql`/* statementTextAndValues */
    SELECT p.id
    FROM unnest(${parentIds}::uuid[], ${rowLimits}::int[])
      AS frontier(parent_id, row_limit)
    CROSS JOIN LATERAL (
      SELECT child.id
      FROM posts child
      WHERE child.parent_id = frontier.parent_id
      ORDER BY child.id
      LIMIT frontier.row_limit
    ) p
  `;
  const { rows } = await read(`${statement.text}`, statement.values);
  return rows;
}
