"use client";
import { Button } from "./Button";

// Client boundary: excluded from results, and the upward RSC chain stops here
// (ClientParent, which imports this, must NOT be reported).
export function ClientThing() {
  return <Button />;
}
