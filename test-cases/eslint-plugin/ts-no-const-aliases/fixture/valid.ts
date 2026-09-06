declare const original: string;
declare const object: { value: string };
declare const values: string[];
declare const items: string[];

// This is a parser-only same-name case; it must not be treated as an alias.
const original = original;
let letAlias = original;
var varAlias = original;
const { value: destructuredAlias } = object;
const [arrayAlias] = values;
const calledAlias = original.trim();
const memberAlias = object.value;
const literalAlias = "original";
const transformedAlias = `${original}`;
const awaitedAlias = await original;
const optionalMemberAlias = object?.value;

for (const item of items) {
  void item;
}

void [
  letAlias,
  varAlias,
  destructuredAlias,
  arrayAlias,
  calledAlias,
  memberAlias,
  literalAlias,
  transformedAlias,
  awaitedAlias,
  optionalMemberAlias,
];
