import sql from 'sql-template-strings';
for (const [name, value] of [['first', 'x'], ['first', 'x'], ['second', 'y']]) {}
for (const [name, value] of [['first', 1]]) {}
for (const [name, value] of [['first', 'x', 'extra']]) {}
for (const [name, value] of [row]) {}
for (const [name, value] of [[...row]]) {}
for (const [name, value] of [[runtimeName, 'x']]) {}
for (const [name, value] of [['first', ...row]]) {}
for (const [name, value, ...rest] of [['first', 'x']]) {}
for (const [, value] of [['first', 'x']]) {}
for (const [name, value] of []) {}
for (const [name, value] of rows) {}
for (const name of ['first', 'first', 'second']) {}
for (const name of []) {}
for (const name of ['first', runtimeName]) {}
for (const name of ['first', ...names]) {}
for (let name of ['first']) {}
for (name of ['first']) {}
for await (const name of ['first']) {}
const yesRoot = sql``.append(`/* ${name} */ SELECT 1`);
const noNonempty = sql`SELECT `.append(`/* ${name} */ SELECT 1`);
const noInterpolated = sql`${prefix}`.append(`/* ${name} */ SELECT 1`);
const noMemberTag = tags.sql``.append(`/* ${name} */ SELECT 1`);
const noSpread = sql``.append(...parts);
const noLiteralName = sql``.append(`/* ${'fixed'} */ SELECT 1`);
const noArguments = sql``.append();
const noMultiple = sql``.append(`/* ${name} */ SELECT 1`, extra);
