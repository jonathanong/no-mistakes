import '../test-helpers/nested-first-live.mts';
test('ordinary nested live helper', async () => {
  await import('../src/nested-target.mts');
});
