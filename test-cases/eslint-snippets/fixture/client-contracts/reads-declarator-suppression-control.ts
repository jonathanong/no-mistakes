// Only the annotated declarator is suppressed; subsequent reads remain visible.
const first = await client.get('a'),
  // no-mistakes-disable-next-line no-consecutive-unbatched-reads
  second = await client.get('b'),
  third = await client.get('c');
await client.get('d');
