import { query } from "@example/db";

// Positional ORDER BY maps to the select list: 1, 2 is account_id, user_id.
export function positional(accountIds: string[], userIds: string[]) {
  return query(
    `INSERT INTO members (account_id, user_id)
     SELECT input.account_id, input.user_id
     FROM unnest($1::uuid[], $2::uuid[]) AS input(account_id, user_id)
     ORDER BY 1, 2
     ON CONFLICT (account_id, user_id) DO NOTHING`,
    [accountIds, userIds],
  );
}

// A bound parameter supplying the leading arbiter column is constant across rows, so ordering
// by it is accepted ...
export function constantLeadInOrder(accountId: string, userIds: string[]) {
  return query(
    `INSERT INTO members (account_id, user_id)
     SELECT $1::uuid, input.user_id
     FROM unnest($2::uuid[]) AS input(user_id)
     ORDER BY $1::uuid, input.user_id
     ON CONFLICT (account_id, user_id) DO NOTHING`,
    [accountId, userIds],
  );
}

// ... and so is leaving it out, because it cannot change the row order.
export function constantLeadOmitted(accountId: string, userIds: string[]) {
  return query(
    `INSERT INTO members (account_id, user_id)
     SELECT $1::uuid, input.user_id
     FROM unnest($2::uuid[]) AS input(user_id)
     ORDER BY input.user_id
     ON CONFLICT (account_id, user_id) DO NOTHING`,
    [accountId, userIds],
  );
}

// Positional reference past a constant leading column.
export function positionalPastConstant(accountId: string, userIds: string[]) {
  return query(
    `INSERT INTO members (account_id, user_id)
     SELECT $1::uuid AS account_id, input.user_id
     FROM unnest($2::uuid[]) AS input(user_id)
     ORDER BY 2
     ON CONFLICT (account_id, user_id) DO NOTHING`,
    [accountId, userIds],
  );
}
