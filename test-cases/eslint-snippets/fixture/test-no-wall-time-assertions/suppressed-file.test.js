// no-mistakes-disable-file test-no-wall-time-assertions -- exercise the real scheduler
expect(Date.now() - start).toBeLessThan(200);
