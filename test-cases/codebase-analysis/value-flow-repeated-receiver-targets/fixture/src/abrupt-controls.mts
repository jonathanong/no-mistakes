import { beforeBreak, beforeContinue, afterReturn, afterThrow, recursive, replica } from '@vendor/valkey';
for (const item of ['a', 'b']) {
  beforeBreak.get(item);
  break;
  beforeBreak.get = replica.get;
}
for (const item of ['a', 'b']) {
  beforeContinue.get(item);
  continue;
  beforeContinue.get = replica.get;
}
function exits() {
  afterReturn.get('key');
  return;
  afterReturn.get = replica.get;
}
exits(); exits();
function throws() {
  afterThrow.get('key');
  throw new Error('stop');
  afterThrow.get = replica.get;
}
try { throws(); throws(); } catch {}
function recurses() {
  recursive.get('key');
  recurses();
  recursive.get = replica.get;
}
recurses(); recurses();
