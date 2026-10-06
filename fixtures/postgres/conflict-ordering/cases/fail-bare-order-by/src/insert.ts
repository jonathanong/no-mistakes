import { query } from "@example/db";

// Both relations declare order_id, so the bare name is ambiguous and stays unmatched.

export function bareAmbiguous(a: string[], b: string[], c: number[]) {
  return query(
    `/* bareAmbiguous */
    INSERT INTO order_lines (order_id, line_no)
    SELECT ids.order_id, nums.line_no
    FROM unnest($1::uuid[]) AS ids(order_id)
    CROSS JOIN unnest($2::uuid[], $3::int[]) AS nums(order_id, line_no)
    ORDER BY order_id, line_no
    ON CONFLICT (order_id, line_no) DO NOTHING`,
    [a, b, c],
  );
}

// A plain table's columns are unknown, so a bare name cannot be attributed to one relation.

export function bareUnknownTable(a: string[]) {
  return query(
    `/* bareUnknownTable */
    INSERT INTO order_lines (order_id, line_no)
    SELECT ids.order_id, nums.line_no
    FROM unnest($1::uuid[]) AS ids(order_id)
    CROSS JOIN line_numbers AS nums
    ORDER BY order_id, line_no
    ON CONFLICT (order_id, line_no) DO NOTHING`,
    [a],
  );
}

// Resolving the bare names must not hide a genuinely reversed order.

export function bareWrongOrder(a: string[], b: number[]) {
  return query(
    `/* bareWrongOrder */
    INSERT INTO order_lines (order_id, line_no)
    SELECT input.order_id, input.line_no
    FROM unnest($1::uuid[], $2::int[]) AS input(order_id, line_no)
    ORDER BY line_no, order_id
    ON CONFLICT (order_id, line_no) DO NOTHING`,
    [a, b],
  );
}
