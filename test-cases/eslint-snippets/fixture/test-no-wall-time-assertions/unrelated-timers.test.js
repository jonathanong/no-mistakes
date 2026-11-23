import { vi } from "./timers.js";

vi.useFakeTimers();
function configure(jest) {
  jest.useFakeTimers();
}
void jest.useFakeTimers;
expect(Date.now() - start).toBeLessThan(200);
