import { forbidden } from './dependency';
export function bridge() {
  if (true) {
    forbidden();
  }
}
// These tests are outside every integration suite; their callbacks are unused.
describe('helper tests', () => {
  test('unused', () => {
    forbidden();
    if (true) {
      forbidden();
    }
  });
});
export const arrow = () => forbidden();
export default function defaultBridge() { forbidden(); }
