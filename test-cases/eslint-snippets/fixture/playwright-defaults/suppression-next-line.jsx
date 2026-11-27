export function SaveButton({ testId }) {
  // no-mistakes-disable-next-line playwright-defaults -- kept for this fixture
  return <button data-pw={testId}>Save</button>;
}
