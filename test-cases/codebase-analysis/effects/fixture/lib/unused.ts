// Not imported from app/server.ts, so its effect call must NOT be reported.
export function dead() {
  invalidate();
}

function invalidate() {}
