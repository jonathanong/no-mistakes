/* eslint-disable react/purity, allowed/rule -- temporary */
// eslint-disable-line react/purity
// eslint-disable-next-line react-hooks-js/refs, react/purity, react/purity
/* oxlint-disable react/purity */
// oxlint-disable-line react-hooks-js/purity
// oxlint-disable-next-line react/purity
/* eslint-disable */
// oxlint-disable-next-line -- blanket reason
// @ts-ignore: intentional unsafe conversion
const x: string = 1;
/* @ts-expect-error explanation */
const y: number = 'a';
// eslint-disable fooish/bar, allowed/rule
// A prose mention of eslint-disable react/purity is not a directive.
/** eslint-disable react/purity */
// eslint-disable: react/purity
const text = '// eslint-disable react/purity';
const template = `/* oxlint-disable react/purity */`;
const pattern = /\/\/ eslint-disable react\/purity/;
const jsx = <div>eslint-disable react/purity{/* oxlint-disable react-hooks-js/refs */}</div>;
