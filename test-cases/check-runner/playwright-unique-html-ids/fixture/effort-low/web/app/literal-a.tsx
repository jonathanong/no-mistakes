import { TEST_CONTROL_ID } from './ids';

export function LiteralA() {
  return (
    <>
      <section id="literal-duplicate">First literal owner</section>
      <button data-testid={TEST_CONTROL_ID}>First test-ID owner</button>
    </>
  );
}
