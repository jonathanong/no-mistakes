import { IMPORTED_ID } from './ids';

function DefaultButton({ testId = 'default-button' }) {
  return <button data-testid={testId}>Default</button>;
}

function makeId() {
  return 'helper-id';
}

export default function Page() {
  const identifierId = 'from-const';
  const flag = true;
  const templateId = 'x';
  return (
    <main>
      <button data-testid="save">Save</button>
      <button data-testid="missing">Missing</button>
      <button data-testid={"publish"}>Publish</button>
      <DefaultButton />
      <button data-testid={identifierId}>From const</button> {/* EXPECT_SHAPE */}
      <button data-testid={flag ? 'ternary-on' : 'ternary-off'}>Ternary</button> {/* EXPECT_SHAPE */}
      <button data-testid={`user-${templateId}`}>Template</button> {/* EXPECT_SHAPE */}
      <button data-testid={IMPORTED_ID}>Imported</button> {/* EXPECT_SHAPE */}
      <button data-testid={makeId()}>Helper</button> {/* EXPECT_SHAPE */}
    </main>
  );
}
