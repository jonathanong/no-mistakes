export const value = fetch('/data', { cache: 'force-cache' }); // no-mistakes-disable-line nextjs-no-caching: used
// no-mistakes-disable-next-line nextjs-no-caching: used
export const other = fetch('/other', { cache: 'force-cache' });
const text = "// no-mistakes-disable-line unknown-in-string";
const template = `
// no-mistakes-disable-next-line unknown-in-template
`;
/*
// no-mistakes-disable-next-line unknown-in-block
*/
