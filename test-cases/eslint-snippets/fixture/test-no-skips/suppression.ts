// no-mistakes-disable-next-line test-no-skips -- explicit exception
it.skip('next line', () => {});
it.only('same line', () => {}); // no-mistakes-disable-line no-mistakes/test-no-skips
// no-mistakes-disable-next-line another-rule
it.todo('still reported');
const text = 'no-mistakes-disable-next-line test-no-skips';
it.fixme('string cannot suppress', () => {});
// no-mistakes-disable-file test-no-skips -- too late to opt out
it.runIf(true)('late', () => {});
/*
 * no-mistakes-disable-next-line test-no-skips
 */ it.skip('block next-line', () => {});
// no-mistakes-disable-next-line
it.only('all rules', () => {});
