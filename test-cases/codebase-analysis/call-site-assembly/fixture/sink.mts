export function sink() { setTimeout(() => {}, 1); }
export class Timer {
  constructor() { sink(); }
  fire() { sink(); }
}
export function make() { return sink; }
export function relay(callback: () => void) { forward(callback); }
function forward(callback: () => void) { callback(); }
