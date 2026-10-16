import { read, receiver } from '@vendor/replacement-demand';
// Suspension and delete invalidate owner proofs before later writes are collected.
export async function awaited() {
  await read;
  receiver.get = read;
}
export function* yielded() {
  yield read;
  receiver.get = read;
}
export function deleted() {
  delete receiver.get;
  receiver.get = read;
}
// Other unary expressions keep the ordinary owner proof available.
export function ordinary() {
  void receiver;
  typeof receiver;
  receiver.get = read;
}
