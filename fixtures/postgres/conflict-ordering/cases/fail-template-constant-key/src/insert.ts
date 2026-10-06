import sql from "sql-template-strings";
import { query } from "@example/db";

// The varying key is not ordered, so the placeholder being constant does not help.

export function controlUnordered(orderId: string, productIds: string[]) {
  return query(sql`/* controlUnordered */
    INSERT INTO order_lines (order_id, product_id)
    SELECT ${orderId}::uuid, input.product_id
    FROM unnest(${productIds}::uuid[]) AS input(product_id)
    ON CONFLICT (order_id, product_id) DO NOTHING
  `);
}

// User text that merely spells a recovery marker is a column, not a bound parameter.

export function userSpelledMarker(a: string[]) {
  return query(
    `/* userSpelledMarker */
    INSERT INTO order_lines (order_id, product_id)
    SELECT sql_placeholder_1, input.product_id
    FROM unnest($1::uuid[]) AS input(product_id)
    ON CONFLICT (order_id, product_id) DO NOTHING`,
    [a],
  );
}

// ... even beside a real interpolation that the recovery names with the same text.

export function userSpelledMarkerBesideReal(orderIds: string[], productId: string) {
  return query(sql`/* userSpelledMarkerBesideReal */
    INSERT INTO order_lines (order_id, product_id)
    SELECT sql_placeholder_1, ${productId}::uuid
    FROM unnest(${orderIds}::uuid[]) AS input(order_id)
    ON CONFLICT (order_id, product_id) DO NOTHING
  `);
}
