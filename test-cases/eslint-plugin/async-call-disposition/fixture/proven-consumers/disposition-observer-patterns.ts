import { enqueueEmail } from '@app/jobs';
// Assignment patterns execute writes; they are not erased binding annotations.
({ expect: globalThis.expect } = { expect: () => ({ resolves: { toBeDefined() {} } }) });
await expect(enqueueEmail()).resolves.toBeDefined();
function objectTarget() {
  const p = enqueueEmail();
  ({ handler: p.catch } = { handler: () => {} });
  p.catch(() => {});
}
function nestedTarget() {
  const p = enqueueEmail();
  ({ handlers: [p.catch = () => {}] } = { handlers: [] });
  p.catch(() => {});
}
function arrayRestTarget() {
  const p = enqueueEmail();
  [,...p.catch] = [undefined, () => {}];
  p.catch(() => {});
}
function objectRestTarget() {
  const p = enqueueEmail();
  ({ ...p.catch } = { handler: () => {} });
  p.catch(() => {});
}
function loopOfTarget(handlers: any[]) {
  const p = enqueueEmail();
  for ([p.catch] of handlers) {}
  p.catch(() => {});
}
function loopInTarget(handlers: any) {
  const p = enqueueEmail();
  for (p.catch in handlers) {}
  p.catch(() => {});
}
function unrelatedPattern() {
  const p = enqueueEmail();
  const box = { value: null };
  ({ value: box.value } = { value: 'safe' });
  p.catch(() => {});
}
function unrelatedLoop() {
  for (const value of []) void value;
}
