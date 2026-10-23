import { TEST_CONTROL_ID } from './ids';

export function LiteralB() {
  return (
    <>
      <aside id="literal-duplicate">Second literal owner</aside>
      <button data-testid={TEST_CONTROL_ID}>Second test-ID owner</button>
    </>
  );
}
