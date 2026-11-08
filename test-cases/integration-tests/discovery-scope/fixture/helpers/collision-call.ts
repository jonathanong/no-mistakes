// No declaration/export names the synthetic function. The escaped local call
// still reaches the historical test callback summary through this helper.
export function callSynthetic() { __te\u0073t_1(); }
test('synthetic target', /* no-mistakes: integration=openai */ () => {});
