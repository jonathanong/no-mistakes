const allowed = ["accounts", "orders"];

export function localWrite(local: string, replacement: string) {
  local = replacement;
  if (!allowed.includes(local)) throw new Error("unknown table");
  return local;
}

export function capturedWrite(captured: string, replacement: string) {
  const reset = () => {
    captured = replacement;
    return false;
  };
  if (!allowed.includes(captured) || reset()) throw new Error("unknown table");
  return captured;
}
