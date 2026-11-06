import sql from 'sql-template-strings';
declare const runtimeFragments: unknown[];
sql`AND true`.append(...runtimeFragments);
