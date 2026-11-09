import SQL from 'sql-template-strings';
import { read } from '@example/db';

export function partialMemberLookup(groupId: number, user: { id: number }) {
  return read(SQL`SELECT COALESCE((SELECT m.role IN ('owner', 'moderator')
    FROM group_members m
    WHERE m.group_id = ${groupId}
      AND m.user_id >= ${user.id}
      AND m.user_id <= ${user.id}
      AND m.removed_at IS NULL), false)
    AND EXISTS (SELECT 1 FROM groups g
      WHERE g.id = ${groupId} AND g.deleted_at IS NULL) AS is_moderator`);
}

export function boundedNoticeTargets(noticeId: number, targetIds: number[]) {
  const statement = SQL`SELECT target.id, party.state
    FROM notice_targets target
    CROSS JOIN LATERAL (`;
  statement.append(SQL`SELECT surface.kind AS state
    FROM placements surface
    WHERE surface.placement_id = target.placement_id`);
  statement.append(SQL` UNION ALL `);
  statement.append(SQL`SELECT recorded.state AS state
    FROM placement_activations recorded
    WHERE recorded.placement_id = target.placement_id
      AND recorded.placement_revision = target.activation_revision`);
  statement.append(SQL` UNION ALL `);
  statement.append(SQL`SELECT g.name AS state
    FROM groups g WHERE g.id = target.notice_id`);
  statement.append(SQL` UNION ALL `);
  statement.append(SQL`SELECT activation.state AS state
    FROM placements surface
    JOIN LATERAL (
      SELECT recorded.state, recorded.bound_by_admin
      FROM placement_activations recorded
      WHERE recorded.placement_id = surface.placement_id
        AND recorded.placement_revision = target.activation_revision
    ) activation ON true
    WHERE surface.placement_id = target.placement_id
      AND surface.kind IN ('comment', 'media')
      AND TRUE AND (FALSE OR activation.bound_by_admin = FALSE)`);
  statement.append(SQL` UNION ALL `);
  statement.append(SQL`SELECT activation.state AS state
    FROM placements surface
    LEFT JOIN LATERAL (
      SELECT recorded.state, recorded.bound_by_admin
      FROM placement_activations recorded
      WHERE recorded.placement_id = surface.placement_id
        AND recorded.placement_revision = target.activation_revision
    ) activation ON true
    JOIN placement_members member
      ON member.placement_id = surface.placement_id
      AND member.member_id = target.member_id
    WHERE surface.placement_id = target.placement_id
      AND surface.kind IN ('comment', 'media')
      AND TRUE AND (FALSE OR activation.bound_by_admin = FALSE)`);
  statement.append(SQL`) party
    WHERE target.notice_id = ${noticeId}
      AND target.id = ANY(${targetIds})`);
  return read(statement);
}

export function missingActivationRevision(noticeId: number, targetIds: number[]) {
  const statement = SQL`SELECT target.id, party.state
    FROM notice_targets target
    CROSS JOIN LATERAL (`;
  statement.append(SQL`SELECT surface.kind AS state FROM placements surface
    WHERE surface.placement_id = target.placement_id`);
  statement.append(SQL` UNION ALL `);
  statement.append(SQL`SELECT recorded.state AS state FROM placement_activations recorded
    WHERE recorded.placement_id = target.placement_id
      AND recorded.placement_revision = target.activation_revision`);
  statement.append(SQL` UNION ALL `);
  statement.append(SQL`SELECT g.name AS state FROM groups g WHERE g.id = target.notice_id`);
  statement.append(SQL` UNION ALL `);
  statement.append(SQL`SELECT activation.state AS state
    FROM placements surface
    JOIN LATERAL (
      SELECT recorded.state, recorded.bound_by_admin
      FROM placement_activations recorded
      WHERE recorded.placement_id = surface.placement_id
    ) activation ON true
    WHERE surface.placement_id = target.placement_id
      AND surface.kind IN ('comment', 'media')
      AND TRUE AND (FALSE OR activation.bound_by_admin = FALSE)`);
  statement.append(SQL` UNION ALL `);
  statement.append(SQL`SELECT activation.state AS state
    FROM placements surface
    LEFT JOIN LATERAL (
      SELECT recorded.state, recorded.bound_by_admin
      FROM placement_activations recorded
      WHERE recorded.placement_id = surface.placement_id
        AND recorded.placement_revision = target.activation_revision
    ) activation ON true
    JOIN placement_members member
      ON member.placement_id = surface.placement_id
      AND member.member_id = target.member_id
    WHERE surface.placement_id = target.placement_id
      AND surface.kind IN ('comment', 'media')
      AND TRUE AND (FALSE OR activation.bound_by_admin = FALSE)`);
  statement.append(SQL`) party WHERE target.notice_id = ${noticeId}
    AND target.id = ANY(${targetIds})`);
  return read(statement);
}

export function nonUniqueMemberArm(noticeId: number, targetIds: number[]) {
  const statement = SQL`SELECT target.id, party.state
    FROM notice_targets target
    CROSS JOIN LATERAL (`;
  statement.append(SQL`SELECT surface.kind AS state FROM placements surface
    WHERE surface.placement_id = target.placement_id`);
  statement.append(SQL` UNION ALL `);
  statement.append(SQL`SELECT recorded.state AS state FROM placement_activations recorded
    WHERE recorded.placement_id = target.placement_id
      AND recorded.placement_revision = target.activation_revision`);
  statement.append(SQL` UNION ALL `);
  statement.append(SQL`SELECT g.name AS state FROM groups g WHERE g.id = target.notice_id`);
  statement.append(SQL` UNION ALL `);
  statement.append(SQL`SELECT activation.state AS state
    FROM placements surface
    JOIN LATERAL (
      SELECT recorded.state, recorded.bound_by_admin
      FROM placement_activations recorded
      WHERE recorded.placement_id = surface.placement_id
        AND recorded.placement_revision = target.activation_revision
    ) activation ON true
    WHERE surface.placement_id = target.placement_id
      AND surface.kind IN ('comment', 'media')
      AND TRUE AND (FALSE OR activation.bound_by_admin = FALSE)`);
  statement.append(SQL` UNION ALL `);
  statement.append(SQL`SELECT activation.state AS state
    FROM placements surface
    LEFT JOIN LATERAL (
      SELECT recorded.state, recorded.bound_by_admin
      FROM placement_activations recorded
      WHERE recorded.placement_id = surface.placement_id
        AND recorded.placement_revision = target.activation_revision
    ) activation ON true
    JOIN placement_members member
      ON member.placement_id = surface.placement_id
    WHERE surface.placement_id = target.placement_id
      AND surface.kind IN ('comment', 'media')
      AND TRUE AND (FALSE OR activation.bound_by_admin = FALSE)`);
  statement.append(SQL`) party WHERE target.notice_id = ${noticeId}
    AND target.id = ANY(${targetIds})`);
  return read(statement);
}

export function nonLateralActivationArm(noticeId: number, targetIds: number[]) {
  const statement = SQL`SELECT target.id, party.state
    FROM notice_targets target CROSS JOIN LATERAL (`;
  statement.append(SQL`SELECT surface.kind AS state FROM placements surface
    WHERE surface.placement_id = target.placement_id`);
  statement.append(SQL` UNION ALL `);
  statement.append(SQL`SELECT recorded.state AS state FROM placement_activations recorded
    WHERE recorded.placement_id = target.placement_id
      AND recorded.placement_revision = target.activation_revision`);
  statement.append(SQL` UNION ALL `);
  statement.append(SQL`SELECT g.name AS state FROM groups g WHERE g.id = target.notice_id`);
  statement.append(SQL` UNION ALL `);
  statement.append(SQL`SELECT activation.state AS state
    FROM placements surface
    JOIN (
      SELECT recorded.state, recorded.bound_by_admin
      FROM placement_activations recorded
      WHERE recorded.placement_id = surface.placement_id
        AND recorded.placement_revision = target.activation_revision
    ) activation ON true
    WHERE surface.placement_id = target.placement_id
      AND surface.kind IN ('comment', 'media')
      AND TRUE AND (FALSE OR activation.bound_by_admin = FALSE)`);
  statement.append(SQL` UNION ALL `);
  statement.append(SQL`SELECT activation.state AS state
    FROM placements surface
    LEFT JOIN LATERAL (
      SELECT recorded.state, recorded.bound_by_admin
      FROM placement_activations recorded
      WHERE recorded.placement_id = surface.placement_id
        AND recorded.placement_revision = target.activation_revision
    ) activation ON true
    JOIN placement_members member
      ON member.placement_id = surface.placement_id
      AND member.member_id = target.member_id
    WHERE surface.placement_id = target.placement_id
      AND surface.kind IN ('comment', 'media')
      AND TRUE AND (FALSE OR activation.bound_by_admin = FALSE)`);
  statement.append(SQL`) party WHERE target.notice_id = ${noticeId}
    AND target.id = ANY(${targetIds})`);
  return read(statement);
}

export function uncorrelatedScalar(noticeId: number) {
  return read(SQL`SELECT (SELECT recorded.state
    FROM placement_activations recorded)
    FROM notice_targets target WHERE target.notice_id = ${noticeId}`);
}
