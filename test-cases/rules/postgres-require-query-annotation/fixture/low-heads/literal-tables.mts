import { read } from '@bench/db';
const STAGES = {
  load: { sql: '/* stages/load */ SELECT 1' },
  save: { sql: `/* stages/save */ SELECT 2` },
  missing: { sql: 'SELECT 3' },
} as const;
read(STAGES.load.sql); // low: true
read(STAGES.save.sql); // low: true
read(STAGES.missing.sql); // low: false
const MUTATED = { load: { sql: '/* mutated/load */ SELECT 1' } } as const;
MUTATED.load.sql = 'SELECT 2';
read(MUTATED.load.sql); // low: false
const ESCAPED = { load: { sql: '/* escaped/load */ SELECT 1' } } as const;
consume(ESCAPED.load);
read(ESCAPED.load.sql); // low: false
const ALIASED = { load: { sql: '/* aliased/load */ SELECT 1' } } as const;
const alias = ALIASED;
read(ALIASED.load.sql); // low: false
const COMPUTED = { load: { sql: '/* computed/load */ SELECT 1' } } as const;
read(COMPUTED['load'].sql); // low: false
export const EXPORTED = { load: { sql: '/* exported/load */ SELECT 1' } } as const;
read(EXPORTED.load.sql); // low: false
const GETTER = { get sql() { return '/* getter */ SELECT 1'; } } as const;
read(GETTER.sql); // low: false
const DYNAMIC = { sql: `/* dynamic */ SELECT ${value}` } as const;
read(DYNAMIC.sql); // low: false
const DUPLICATE = { sql: '/* first */ SELECT 1', sql: 'SELECT 2' } as const;
read(DUPLICATE.sql); // low: false
const SPREAD = { ...other, sql: '/* spread */ SELECT 1' } as const;
read(SPREAD.sql); // low: false
const UNASSERTED = { sql: '/* unasserted */ SELECT 1' };
read(UNASSERTED.sql); // low: false
function shadow(STAGES) {
  read(STAGES.load.sql); // low: false
}
const OPTIONAL = { sql: '/* optional */ SELECT 1' } as const;
read?.(OPTIONAL.sql); // low: false
const OPTIONAL_MEMBER = { sql: '/* optional/member */ SELECT 1' } as const;
read(OPTIONAL_MEMBER?.sql); // low: false
const UNKNOWN_PATH = { sql: '/* existing */ SELECT 1' } as const;
read(UNKNOWN_PATH.absent); // low: false
const PRODUCED = { sql: makeSql() } as const;
read(PRODUCED.sql); // low: false
const PROTOTYPE = { __proto__: other, sql: '/* proto */ SELECT 1' } as const;
read(PROTOTYPE.sql); // low: false
const QUOTED = { 'sql': '/* quoted */ SELECT 1' } as const;
read(QUOTED.sql); // low: false
const MULTILINE = { sql: '/* multiline */ SELECT 1' } as
const;
read(MULTILINE.sql); // low: true
const { sql: extracted } = other;
const AS_STRING = { sql: '/* string */ SELECT 1' } as string;
read(AS_STRING.sql); // low: false
const AS_NAMED = { sql: '/* named */ SELECT 1' } as Named;
read(AS_NAMED.sql); // low: false
const AS_QUALIFIED = { sql: '/* qualified */ SELECT 1' } as types.Named;
read(AS_QUALIFIED.sql); // low: false
const ROOT_LITERAL = '/* literal */ SELECT 1' as const;
read(ROOT_LITERAL); // low: true
const METHOD = { sql() { return '/* method */ SELECT 1'; } } as const;
read(METHOD.sql); // low: false
const SHORTHAND = { extracted } as const;
read(SHORTHAND.extracted); // low: false
const COMPUTED_PROPERTY = { ['sql']: '/* computed/property */ SELECT 1' } as const;
read(COMPUTED_PROPERTY.sql); // low: false
