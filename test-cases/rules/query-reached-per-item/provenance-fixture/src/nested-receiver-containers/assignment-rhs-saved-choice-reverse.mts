import { primary, replica } from '@vendor/valkey';
function replacement() {}
const box = { wrapper: {} };
const retained = { client: primary };
const other = { client: replica };
box.wrapper = retained;
const before = box.wrapper;
box.wrapper = other;
const after = box.wrapper;
const choices = [after, before];
choices[globalThis.key].client.get = replacement;

function consume(value, other) { value.get('original'); other.get('changed'); }
consume(primary, replica);
