import * as deps from "@app/jobs";
import { enqueueEmail } from "@app/jobs";
declare const cond: boolean;
declare function unknown(value: unknown): unknown;
export async function examples() {
  await Promise.all([...(cond ? [deps.enqueueEmail()] : [])]);
  await Promise.all([...(cond ? [] : [enqueueEmail()])]);
  await Promise.all([...(cond ? [...[enqueueEmail()]] : [])]);
  await Promise.all([cond ? enqueueEmail() : enqueueEmail()]);
  await Promise.all([...(cond ? [1].map(() => enqueueEmail()) : [])]);
  Promise.all([...(cond ? [deps.enqueueEmail()] : [])]); // dropped
  await Promise.all([[enqueueEmail()]]); // nested array
  await Promise.all([[1].map(() => enqueueEmail())]); // nested map
  await Promise.all([...(cond ? [[enqueueEmail()]] : [])]); // spread does not flatten members
  await Promise.all([unknown([enqueueEmail()])]); // unknown consumer
  await Promise.all([{ value: enqueueEmail() }]); // object escape
  await Promise.all([enqueueEmail() ? 1 : 2]); // condition
  await Promise.allSettled([...(cond ? [deps.enqueueEmail()] : [])]);
  await Promise.allSettled([...(cond ? [] : [enqueueEmail()])]);
  await Promise.allSettled([...(cond ? [...[enqueueEmail()]] : [])]);
  await Promise.allSettled([cond ? enqueueEmail() : enqueueEmail()]);
  await Promise.allSettled([...(cond ? [1].map(() => enqueueEmail()) : [])]);
  Promise.allSettled([...(cond ? [deps.enqueueEmail()] : [])]); // dropped
  await Promise.allSettled([[enqueueEmail()]]); // nested array
  await Promise.allSettled([[1].map(() => enqueueEmail())]); // nested map
  await Promise.allSettled([...(cond ? [[enqueueEmail()]] : [])]); // spread does not flatten members
  await Promise.allSettled([unknown([enqueueEmail()])]); // unknown consumer
  await Promise.allSettled([{ value: enqueueEmail() }]); // object escape
  await Promise.allSettled([enqueueEmail() ? 1 : 2]); // condition
  await Promise.race([...(cond ? [deps.enqueueEmail()] : [])]);
  await Promise.race([...(cond ? [] : [enqueueEmail()])]);
  await Promise.race([...(cond ? [...[enqueueEmail()]] : [])]);
  await Promise.race([cond ? enqueueEmail() : enqueueEmail()]);
  await Promise.race([...(cond ? [1].map(() => enqueueEmail()) : [])]);
  Promise.race([...(cond ? [deps.enqueueEmail()] : [])]); // dropped
  await Promise.race([[enqueueEmail()]]); // nested array
  await Promise.race([[1].map(() => enqueueEmail())]); // nested map
  await Promise.race([...(cond ? [[enqueueEmail()]] : [])]); // spread does not flatten members
  await Promise.race([unknown([enqueueEmail()])]); // unknown consumer
  await Promise.race([{ value: enqueueEmail() }]); // object escape
  await Promise.race([enqueueEmail() ? 1 : 2]); // condition
  await Promise.any([...(cond ? [deps.enqueueEmail()] : [])]);
  await Promise.any([...(cond ? [] : [enqueueEmail()])]);
  await Promise.any([...(cond ? [...[enqueueEmail()]] : [])]);
  await Promise.any([cond ? enqueueEmail() : enqueueEmail()]);
  await Promise.any([...(cond ? [1].map(() => enqueueEmail()) : [])]);
  Promise.any([...(cond ? [deps.enqueueEmail()] : [])]); // dropped
  await Promise.any([[enqueueEmail()]]); // nested array
  await Promise.any([[1].map(() => enqueueEmail())]); // nested map
  await Promise.any([...(cond ? [[enqueueEmail()]] : [])]); // spread does not flatten members
  await Promise.any([unknown([enqueueEmail()])]); // unknown consumer
  await Promise.any([{ value: enqueueEmail() }]); // object escape
  await Promise.any([enqueueEmail() ? 1 : 2]); // condition
  await Promise.all([1].map(() => [enqueueEmail()])); // mapper array stays nested
  await Promise.all([...[1].map(() => [enqueueEmail()])]); // spread preserves mapper members
  deps.enqueueEmail(); // direct imported namespace drop
  enqueueEmail(); // direct import drop
  // Unknown transformations and logical paths must not prove aggregate adoption.
  await Promise.all([...(cond ? [enqueueEmail() + 1] : [])]);
  await Promise.all([...(cond ? [(enqueueEmail(), 0)] : [])]);
  await Promise.all([...(cond ? [typeof enqueueEmail()] : [])]);
  await Promise.all([...(cond ? [enqueueEmail() && 0] : [])]);
  await Promise.all([...(cond ? [enqueueEmail() || 0] : [])]);
  await Promise.all([...(cond ? [enqueueEmail() ?? 0] : [])]);
  await Promise.all([...(cond ? [0 && enqueueEmail()] : [])]);
  await Promise.all([...(cond ? [!enqueueEmail()] : [])]);
  await Promise.all([...(cond ? [`${enqueueEmail()}`] : [])]);
  await Promise.all([...(cond ? [(cond = enqueueEmail())] : [])]);
}
