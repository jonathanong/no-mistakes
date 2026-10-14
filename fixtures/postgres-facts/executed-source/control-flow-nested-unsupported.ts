import { read, sql } from '@example/db';

function mutateInSelectedArm(query: unknown) {
  if (true) {
    for (const unused of [1]) query.append(sql`; DELETE FROM records`);
  }
}
function mutateInReturnedArm(query: unknown) {
  if (true) {
    for (const unused of [1]) query.append(sql`; DELETE FROM records`);
    return 'known final value';
  }
}
function skippedUnsupportedArm(query: unknown) {
  // An unreachable unsupported arm must not poison the selected safe path.
  if (false) {
    for (const unused of [1]) query.append(sql`; DELETE FROM records`);
  }
}
export function selectedUnsupportedArm() {
  const query = sql`/* nested/selected */ SELECT 1`;
  mutateInSelectedArm(query);
  read(query);
}
export function returnedUnsupportedArm() {
  const query = sql`/* nested/returned */ SELECT 1`;
  mutateInReturnedArm(query);
  read(query);
}
export function unreachableUnsupportedArm() {
  const query = sql`/* nested/unreachable */ SELECT 1`;
  skippedUnsupportedArm(query);
  read(query);
}
