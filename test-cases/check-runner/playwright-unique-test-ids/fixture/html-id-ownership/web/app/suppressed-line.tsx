const silentId = 'suppressed-line-id';

export function SuppressedLine() {
  // no-mistakes-disable-next-line playwright-unique-test-ids
  return <button data-testid={silentId} />;
}
