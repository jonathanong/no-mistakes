import { query } from "@example/db";

// Each statement still locks more than one row. Do not "fix" the rule by accepting them.

// The join is on a non-key column of the locked relation.
export function joinOnNonKey(tokenHash: string, uris: string[]) {
  return query(
    `SELECT app_grant.id FROM auth_codes AS code
     JOIN grants AS app_grant ON app_grant.account_id = code.account_id
     WHERE code.token_hash = $1 AND app_grant.resource = ANY($2)
     FOR UPDATE OF app_grant`,
    [tokenHash, uris],
  );
}

// A LEFT JOIN equality is not used as a pin.
export function leftJoin(tokenHash: string, uris: string[]) {
  return query(
    `SELECT app_grant.id FROM auth_codes AS code
     LEFT JOIN grants AS app_grant ON app_grant.id = code.grant_id
     WHERE code.token_hash = $1 AND app_grant.resource = ANY($2)
     FOR UPDATE OF app_grant`,
    [tokenHash, uris],
  );
}

// The join equality sits inside an OR.
export function orWrappedJoin(tokenHash: string, uris: string[]) {
  return query(
    `SELECT app_grant.id FROM auth_codes AS code
     JOIN grants AS app_grant ON (app_grant.id = code.grant_id OR app_grant.account_id = code.account_id)
     WHERE code.token_hash = $1 AND app_grant.resource = ANY($2)
     FOR UPDATE OF app_grant`,
    [tokenHash, uris],
  );
}

// The anchor relation is not pinned, so nothing is single-row.
export function unpinnedAnchor(uris: string[]) {
  return query(
    `SELECT app_grant.id FROM auth_codes AS code
     JOIN grants AS app_grant ON app_grant.id = code.grant_id
     WHERE code.redirect_uri = ANY($1)
     FOR UPDATE OF code, app_grant`,
    [uris],
  );
}

// A user-authored identifier that only spells the interpolation marker is a column,
// not a bind, in a plain string.
export function userAuthoredMarker(uris: string[]) {
  return query(
    `SELECT code.id FROM auth_codes AS code
     WHERE code.id = sql_placeholder_x AND code.redirect_uri = ANY($1)
     FOR UPDATE`,
    [uris],
  );
}
