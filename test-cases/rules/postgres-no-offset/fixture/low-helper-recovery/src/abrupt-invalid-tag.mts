import SQL from 'sql-template-strings';
import { read } from '@example/db';

function sourceCtor(_value: unknown) {
  read('DROP TABLE source-visible');
}
function selected(kind: string) {
  switch (kind) {
    case 'safe': return 'SELECT 1';
    default: throw new sourceCtor(SQL`ignored`);
  }
}
function external(strings: TemplateStringsArray) { return strings[0]; }
// This later write invalidates the constructor argument's tag value proof.
SQL = external;
export function sample(kind: string) {
  read(selected(kind)); // RESIDUAL: the throw-side constructor still calls SQL.
}
