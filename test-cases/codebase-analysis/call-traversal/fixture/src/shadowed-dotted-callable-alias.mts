function target() {}
function other() {}

export function callThroughShadow() {
  const invoke = target;
  const calls = { run: invoke };
  {
    const invoke = other;
    // Object captured the outer `invoke`; inner shadow must not steal the hop.
    calls.run();
  }
}
