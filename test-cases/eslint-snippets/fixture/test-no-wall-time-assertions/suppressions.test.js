// no-mistakes-disable-next-line test-no-wall-time-assertions -- temporarily track a real timer
expect(Date.now() - start).toBeLessThan(200);
expect(Date.now() - start).toBeLessThan(200); // no-mistakes-disable-line test-no-wall-time-assertions -- same timer
expect(Date.now() - start).toBeLessThan(200);
