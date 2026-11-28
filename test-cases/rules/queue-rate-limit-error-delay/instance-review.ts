declare function clearDelay(error: object): void;

export function reviewFixes(flag: boolean): void {
  const zeroAfter = Object.assign(new Error(), { delayMs: 5000 });
  zeroAfter.delayMs = 0;
  if (flag) throw zeroAfter;

  const earlyThrow = new Error();
  if (flag) throw earlyThrow;
  earlyThrow.delayMs = 5000;
  throw earlyThrow;

  const escaped = new Error();
  escaped.delayMs = 1000;
  clearDelay(escaped);
  if (flag) throw escaped;

  const restored = new Error();
  clearDelay(restored);
  restored.delayMs = 1000;
  if (flag) throw restored;

  const updated = new Error();
  updated.delayMs = 5000;
  updated.delayMs++;
  if (flag) throw updated;

  const deleted = new Error();
  deleted.delayMs = 5000;
  delete deleted.delayMs;
  if (flag) throw deleted;

  const conditional = new Error();
  conditional.delayMs = 5000;
  if (flag) conditional.delayMs = 0;
  if (flag) throw conditional;

  const held = new Error();
  held.delayMs = 5000;
  throw held;

  const nestedThrow = new Error();
  const failNested = () => {
    throw nestedThrow;
  };
  nestedThrow.delayMs = 1000;
  failNested();

  const nestedAssign = Object.assign(
    Object.assign(new Error(), { delayMs: 1000 }),
    { delayMs: 0 },
  );
  if (flag) throw nestedAssign;
}
