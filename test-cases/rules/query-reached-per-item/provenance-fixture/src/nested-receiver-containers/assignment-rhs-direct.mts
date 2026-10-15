import { primary, replica } from '@vendor/valkey';
function replacement() {}
const box = { wrapper: primary };
box.wrapper = replica;
box.wrapper.get = replacement;
function consume(value, changed) { value.get('original'); changed.get('changed'); }
consume(primary, replica);
