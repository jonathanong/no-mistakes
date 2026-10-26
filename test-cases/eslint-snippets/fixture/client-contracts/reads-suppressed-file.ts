// no-mistakes-disable-file no-consecutive-unbatched-reads
await client.get('a');
await client.get('b');
