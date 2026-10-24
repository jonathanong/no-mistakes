const begin = "BEGIN";
const alias = begin;
// Dynamic environments do not establish same-scope reads of the alias.
with (environment) { client.query(alias); }
function evaluate() { eval("var alias = 'SELECT 1'"); client.query(alias); }
