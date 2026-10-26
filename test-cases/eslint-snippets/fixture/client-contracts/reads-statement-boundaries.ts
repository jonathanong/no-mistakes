import { cache as client } from '@app/cache';
let key, other, rest;
key = await client.get('a');
await client.get(key);
;
({ key, ...rest } = await client.get('a'));
await client.get(rest.next);
;
[key = 'fallback', ...rest] = await client.get('a');
await client.get(key);
;
const first = await client.get('a'), second = await client.get(first.next), barrier = other(), third = await client.get('b');
;
const fourth = await client.get('a'), fifth = await client.get(fourth.next), sixth = await client.get(fifth.next);
;
switch (kind) {
  // Fallthrough does not make separate cases one unconditional sequence.
  case 'one':
    await client.get('a');
  case 'two':
    await client.get('b');
}
;
key += await client.get('a');
await client.get('b');
;
container.key = await client.get('a');
await other.get('b');
;
export { key };
await client.get('c');
;
// Property result dependencies cannot be resolved as local bindings.
container.key = await client.get('a');
await client.get(container.key);
;
[container.key] = await client.get('a');
await client.get(container.key);
;
unbound = await client.get('a');
await client.get(unbound);
;
// Both the declarator and assignment receive this result.
const assignedDeclaration = key = await client.get('a');
await client.get(key);
