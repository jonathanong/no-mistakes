import { it } from 'vitest';

// Call-site arguments do not prove the parameter's value at registration.
function run(ms: number) {
  it('runs', () => {}, ms);
}
run(60_000);
