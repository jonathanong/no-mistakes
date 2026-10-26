await client.get('a');
// no-mistakes-disable-next-line no-consecutive-unbatched-reads
await client.get('b');
