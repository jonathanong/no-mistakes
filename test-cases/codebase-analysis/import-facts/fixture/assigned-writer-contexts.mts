import { primary, session } from '@vendor/valkey';
const first = primary;
const second = primary;
function writeFirst() {
  first.get = replacement;
  first[key] = replacement;
  session.info = replacement;
  session.info = replacement;
}
function writeSecond() {
  second.get = replacement;
  second[key] = replacement;
}
