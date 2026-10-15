import { primary, replica, untouched } from '@vendor/valkey';
// A single call before its later write retains the original target.
primary.get('once');
primary.get = replica.get;
// A method acquired once remains a snapshot during later repeated execution.
const saved = untouched.get;
for (const item of ['a', 'b']) {
  saved(item);
  untouched.get = replica.get;
  untouched.set(item, 'value');
}
// Confirms the replacement-only selector and per-item rule are active.
for (const item of ['a', 'b']) {
  replica.get(item);
}
