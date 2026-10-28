import sql from 'sql-template-strings'
export function appendImported(options, builder) { builder.append(sql` WHERE enabled`); }
