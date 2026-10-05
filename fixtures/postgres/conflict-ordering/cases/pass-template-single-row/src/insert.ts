import sql from "sql-template-strings";
import { query } from "@example/db";

// Each `${...}` interpolation is a bound value, so this FROM-less SELECT is one row whatever its
// WHERE says.
export function addDelivery(input: {
  orderId: string;
  submissionId: string | null;
  messageId: string | null;
  recipientAccountId: string;
  recipientRole: string;
  kind: string;
  targetPath?: string;
  channel: string;
  idempotencyKey: string;
}) {
  return query(sql`
    INSERT INTO order_deliveries (order_id, submission_id, message_id, recipient_account_id, recipient_role, kind, target_path, channel, idempotency_key)
    SELECT ${input.orderId}, ${input.submissionId}, ${input.messageId}, ${input.recipientAccountId}, ${input.recipientRole}, ${input.kind}, ${input.targetPath ?? null}, ${input.channel}, ${input.idempotencyKey}
    WHERE EXISTS (SELECT 1 FROM orders WHERE id = ${input.orderId})
      AND (${input.submissionId}::uuid IS NULL OR EXISTS (SELECT 1 FROM order_submissions WHERE id = ${input.submissionId} AND order_id = ${input.orderId}))
      AND (${input.messageId}::uuid IS NULL OR EXISTS (SELECT 1 FROM order_messages WHERE id = ${input.messageId} AND order_id = ${input.orderId}))
    ON CONFLICT (idempotency_key) DO NOTHING
    RETURNING id, order_id, state
  `);
}

// CURRENT_TIMESTAMP is a scalar, and the ON CONFLICT action may use more interpolations.
export function touchSession(options: { sid: string; deviceId: string; expiresAt: Date }, accountId: string, refreshMetadata: boolean) {
  return query(sql`
    INSERT INTO sessions (id, account_id, device_id, expires_at, last_seen_at)
    SELECT ${options.sid}, ${accountId}, ${options.deviceId}, ${options.expiresAt}, CURRENT_TIMESTAMP
    WHERE EXISTS (SELECT 1 FROM accounts WHERE id = ${accountId} AND deleted_at IS NULL)
    ON CONFLICT (id) DO UPDATE SET account_id = EXCLUDED.account_id,
      device_id = CASE WHEN ${refreshMetadata} THEN EXCLUDED.device_id ELSE sessions.device_id END,
      expires_at = EXCLUDED.expires_at, last_seen_at = CURRENT_TIMESTAMP
    WHERE sessions.revoked_at IS NULL
    RETURNING id, account_id
  `);
}
