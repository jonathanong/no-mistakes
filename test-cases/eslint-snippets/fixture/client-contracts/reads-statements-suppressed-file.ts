// no-mistakes-disable-file no-consecutive-unbatched-reads
let a;
a = await client.get('a');
a = await client.get('b');
export const first = await client.get('a'), second = await client.get('b');
async function returned() {
  await client.get('a');
  return await client.get('b');
}
switch (kind) {
  case 'one':
    await client.get('a');
    await client.get('b');
}
