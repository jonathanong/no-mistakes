import { mkdtempSync, readdirSync } from 'node:fs';
import { dirname } from 'node:path';
function both(flag) {
  if (flag) { return mkdtempSync('left-'); } else { return mkdtempSync('right-'); }
}
function loop(items) {
  for (const item of items) { if (item) { return mkdtempSync('item-'); } }
  return mkdtempSync('fallback-');
}
function parent(flag) {
  if (flag) { return dirname(mkdtempSync('left-')); } else { return dirname(mkdtempSync('right-')); }
}
function incomplete(flag) {
  if (flag) { return mkdtempSync('partial-'); }
}
function unsupported(items) {
  for (const item of items) { break; }
  return mkdtempSync('after-');
}
function expression(flag) { return flag ? mkdtempSync('yes-') : mkdtempSync('no-'); }
export function inspect(flag, items) {
  readdirSync(both(flag), 'both');
  readdirSync(loop(items), 'loop');
  readdirSync(parent(flag), 'parent');
  readdirSync(incomplete(flag), 'incomplete');
  readdirSync(unsupported(items), 'unsupported');
  readdirSync(expression(flag), 'expression');
}
