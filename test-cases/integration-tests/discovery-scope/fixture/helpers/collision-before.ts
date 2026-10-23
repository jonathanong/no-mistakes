// Synthetic callback summaries historically overwrite this exported helper.
export function __test_1() {}
test('overwrites helper', /* no-mistakes: integration=openai */ () => {});
