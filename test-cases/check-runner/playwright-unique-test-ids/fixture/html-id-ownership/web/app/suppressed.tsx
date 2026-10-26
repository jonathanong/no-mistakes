// no-mistakes-disable-file playwright-unique-test-ids
// no-mistakes-disable-file playwright-unique-html-ids
const silentId = 'suppressed-file-id';

export function Suppressed() {
  return <button id={silentId} data-testid={silentId} />;
}
