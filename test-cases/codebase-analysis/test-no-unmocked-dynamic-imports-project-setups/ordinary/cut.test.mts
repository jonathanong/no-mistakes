import '../test-helpers/nested-first.mts';
test('ordinary nested cut', async () => {
  await import('../src/nested-target.mts');
});
