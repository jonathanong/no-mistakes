let a, b;
a = await client.get('a');
// no-mistakes-disable-next-line no-consecutive-unbatched-reads
b = await client.get('b');
;
export const first = await client.get('a');
export const second = await client.get('b'); // no-mistakes-disable-line no-consecutive-unbatched-reads
async function returned() {
  await client.get('a');
  // no-mistakes-disable-next-line no-consecutive-unbatched-reads
  return await client.get('b');
}
switch (kind) {
  case 'one':
    await client.get('a');
    // no-mistakes-disable-next-line no-consecutive-unbatched-reads
    await client.get('b');
}
;
const third = await client.get('a'),
  // no-mistakes-disable-next-line no-consecutive-unbatched-reads
  fourth = await client.get('b');
;
// A directive on the declaration header covers later declarators too.
// no-mistakes-disable-next-line no-consecutive-unbatched-reads
const headerA = await client.get('a'),
  headerB = await client.get('b'),
  headerC = await client.get('c');
;
// no-mistakes-disable-next-line no-consecutive-unbatched-reads
export const exportA = await client.get('a'),
  exportB = await client.get('b');
;
// The export header owns the declaration even when const starts on another line.
// no-mistakes-disable-next-line no-consecutive-unbatched-reads
export
const splitA = await client.get('a'),
  splitB = await client.get('b');
