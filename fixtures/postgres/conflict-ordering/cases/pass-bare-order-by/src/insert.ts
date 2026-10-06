import { query } from "@example/db";

// A bare ORDER BY column names the qualified select expression when `input` is the only relation.

export function bareSingleRelation(a: string[], b: number[]) {
  return query(
    `/* bareSingleRelation */
    INSERT INTO order_lines (order_id, line_no)
    SELECT input.order_id, input.line_no
    FROM unnest($1::uuid[], $2::int[]) AS input(order_id, line_no)
    ORDER BY order_id, line_no
    ON CONFLICT (order_id, line_no) DO NOTHING`,
    [a, b],
  );
}

// Two relations, but only one declares each column, so every bare name is unambiguous.

export function bareDerivedColumns(a: string[], b: number[]) {
  return query(
    `/* bareDerivedColumns */
    INSERT INTO order_lines (order_id, line_no)
    SELECT ids.order_id, nums.line_no
    FROM unnest($1::uuid[]) AS ids(order_id)
    CROSS JOIN unnest($2::int[]) AS nums(line_no)
    ORDER BY order_id, line_no
    ON CONFLICT (order_id, line_no) DO NOTHING`,
    [a, b],
  );
}
