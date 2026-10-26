import { IMPORTED_ID } from './ids';
import { OtherSave } from './other';

const SILENT_ID = 'silent-id';
const flag = true;
const templateId = 'x';

function makeId() {
  return 'helper-id';
}

function DefaultButton({ testId = 'default-button' }) {
  return <button data-testid={testId}>Default</button>;
}

function Suppressed() {
  // no-mistakes-disable-next-line playwright-unique-test-ids
  return <button data-testid={SILENT_ID}>Suppressed</button>;
}

export default function Page() {
  const identifierId = 'from-const';
  return (
    <main>
      <button data-testid="save">Save</button>
      <button data-testid={"publish"}>Publish</button>
      <DefaultButton />
      <Suppressed />
      <OtherSave />
      <button data-testid={identifierId}>From const</button> {/* EXPECT_SHAPE */}
      <button data-testid={flag ? 'ternary-on' : 'ternary-off'}>Ternary</button> {/* EXPECT_SHAPE */}
      <button data-testid={flag && 'logical-id'}>Logical</button> {/* EXPECT_SHAPE */}
      <button data-testid={`user-${templateId}`}>Template</button> {/* EXPECT_SHAPE */}
      <button data-testid={IMPORTED_ID}>Imported</button> {/* EXPECT_SHAPE */}
      <button data-testid={makeId()}>Helper</button> {/* EXPECT_SHAPE */}
    </main>
  );
}
