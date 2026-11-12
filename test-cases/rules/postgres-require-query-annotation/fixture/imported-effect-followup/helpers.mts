import sql from 'sql-template-strings';
export function addFilter(builder) { builder.append(sql` WHERE active`); }
export function rewriteHead(builder) { builder.strings[0] = 'SELECT 1'; }
