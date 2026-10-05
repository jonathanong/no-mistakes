import { query } from "@example/db";

// `code` is pinned by its unique token_hash. `app_grant` is pinned through the
// inner-join equality on its primary key to that single-row relation, and the
// `= ANY` compares against a joined row's array column, not a key list.
// FOR SHARE is not an update lock.
export function lockCode(tokenHash: string) {
  return query(
    `SELECT code.id, app_grant.client_id, client.owner_account_id
     FROM auth_codes AS code
     JOIN grants AS app_grant ON app_grant.id = code.grant_id
     JOIN clients AS client ON client.id = app_grant.client_id
     WHERE code.token_hash = $1
       AND client.revoked_at IS NULL
       AND code.redirect_uri = ANY(client.redirect_uris)
       AND NOT EXISTS (SELECT 1 FROM suspensions AS suspension WHERE suspension.account_id = app_grant.account_id AND suspension.lifted_at IS NULL)
     FOR UPDATE OF code, app_grant FOR SHARE OF client`,
    [tokenHash],
  );
}

// Interpolated values are binds; the OR is on unlocked relations' filters only.
export function lockSubscription(subscriptionId: string, accountId: string) {
  return query(
    `SELECT subscription.id
     FROM subscriptions subscription
     INNER JOIN subscription_sources source ON source.id = subscription.source_id
     INNER JOIN subscription_source_states source_state ON source_state.source_id = source.id
     WHERE subscription.id = ${subscriptionId} AND subscription.account_id = ${accountId}
       AND subscription.ended_at IS NULL
       AND ((source.source_kind IN ('direct', 'family') AND source_state.cancelled_at IS NOT NULL)
         OR (source.source_kind = 'admin_grant' AND subscription.cancelled_at IS NOT NULL))
     FOR UPDATE OF subscription`,
    [],
  );
}
