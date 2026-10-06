import sql from "sql-template-strings";
import { query } from "@example/db";

// A recovered `${...}` placeholder is a bound parameter, bare or cast, leading or trailing.

export function leadingCast(orderId: string, productIds: string[]) {
  return query(sql`/* leadingCast */
    INSERT INTO order_lines (order_id, product_id)
    SELECT ${orderId}::uuid, input.product_id
    FROM unnest(${productIds}::uuid[]) AS input(product_id)
    ORDER BY input.product_id
    ON CONFLICT (order_id, product_id) DO NOTHING
  `);
}

export function leadingBare(orderId: string, productIds: string[]) {
  return query(sql`/* leadingBare */
    INSERT INTO order_lines (order_id, product_id)
    SELECT ${orderId}, input.product_id
    FROM unnest(${productIds}::uuid[]) AS input(product_id)
    ORDER BY input.product_id
    ON CONFLICT (order_id, product_id) DO NOTHING
  `);
}

export function trailingCast(orderIds: string[], productId: string) {
  return query(sql`/* trailingCast */
    INSERT INTO order_lines (order_id, product_id)
    SELECT input.order_id, ${productId}::uuid
    FROM unnest(${orderIds}::uuid[]) AS input(order_id)
    ORDER BY input.order_id
    ON CONFLICT (order_id, product_id) DO NOTHING
  `);
}
