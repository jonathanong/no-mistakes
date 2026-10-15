import { foreignOnly, localWriter, nestedWriter, defaultReceiver, lexical, lexicalEarly, hot, rhs } from '@vendor/source-timing';
import { forward } from './helper.mts';
const savedForeign = foreignOnly.get;
function copyLocal() {
  const get = localWriter.get;
  localWriter.get = rhs.get;
  return get;
}
const localSaved = copyLocal();
localSaved('outside');
function copyNested() {
  const get = nestedWriter.get;
  function innerCopy() { return get; }
  nestedWriter.get = rhs.get;
  return innerCopy();
}
const nestedSaved = copyNested();
nestedSaved('outside');
function copyDefault(client = defaultReceiver, get = client.get) {
  defaultReceiver.get = rhs.get;
  return get;
}
const defaultSaved = copyDefault();
defaultSaved('outside');
// An unsupported repeated writer prevents a stable entry snapshot.
while (active) { lexicalEarly.get = rhs.get; }
for (const item of ['a', 'b']) {
  lexical.info(item);
  lexicalEarly.get(item);
  foreignOnly.get(item);
  foreignOnly.set(item, 'value');
  savedForeign(item);
  const nestedRepeated = copyNested();
  nestedRepeated(item);
  const repeatedSaved = copyLocal();
  repeatedSaved(item);
  const forwarded = forward();
  forwarded(item);
  hot.get = rhs.get;
}
// This later lexical write does not run between the earlier loop's reads.
lexical[key].get = rhs.get;
