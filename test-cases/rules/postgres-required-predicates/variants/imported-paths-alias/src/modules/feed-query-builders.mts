export const feedVisibleSql = (ownerCol: string): string =>
  `EXISTS (SELECT 1 FROM view_active_plans p WHERE p.account_id = ${ownerCol})`;
