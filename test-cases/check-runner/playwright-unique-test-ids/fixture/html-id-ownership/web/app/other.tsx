import { SHARED_ID } from './ids';

export function Other() {
  return (
    <>
      <div id="literal-id" />
      <div id={SHARED_ID} /> {/* EXPECT_HTML */}
      <button data-testid="literal-test" />
    </>
  );
}
