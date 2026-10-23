await client.get('a');
await client.get('b'); // no-mistakes-disable-line no-consecutive-unbatched-reads
