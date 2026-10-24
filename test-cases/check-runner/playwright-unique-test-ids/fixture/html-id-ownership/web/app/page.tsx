import { SHARED_ID, TEST_ID } from './ids';

function makeId() {
  return 'helper-html-id';
}

export default function Page() {
  return (
    <main>
      <div id="literal-id" />
      <div id={SHARED_ID} /> {/* EXPECT_HTML */}
      <div id={makeId()} /> {/* EXPECT_HTML */}
      <button data-testid="literal-test" />
      <button data-testid={TEST_ID} /> {/* EXPECT_TEST */}
    </main>
  );
}
