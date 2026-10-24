// Exported aliases can expose synthetic callback summaries without declarations.
export { __test_1 as exportedSynthetic };
test('exported synthetic', /* no-mistakes: integration=openai */ () => {});
