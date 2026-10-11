import '../test-helpers/z-sibling-cutter.mts';
import '../test-helpers/a-sibling-mocker.mts';
test('ordinary lexical sibling cut', async () => {
  await import('../src/sibling-target.mts');
});
