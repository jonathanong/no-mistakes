import { primary, session } from '@vendor/valkey';
const items = ['one'];
// A literal star is unrelated to the selected client property.
const literal = { client: primary, '*': {} };
function literalStar({ client } = literal) { literal['*'].get = unrelated; client.get('key'); }
for (const item of items) { literalStar(); }
const sameStar = { '*': session };
function selectedStar({ '*': client } = sameStar) { sameStar['*'].info = unrelated; client.info('key'); }
for (const item of items) { selectedStar(); }
const dotted = { 'a.b': primary };
function dottedKey({ 'a.b': client } = dotted) { dotted['a.b'].get = unrelated; client.get('key'); }
for (const item of items) { dottedKey(); }
const computed = { client: primary };
function dynamicKey({ client } = computed) { computed[key].get = unrelated; client.get('key'); }
for (const item of items) { dynamicKey(); }
