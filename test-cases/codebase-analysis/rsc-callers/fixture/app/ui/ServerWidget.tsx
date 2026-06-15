"use server";
import { Button } from "./Button";

// Explicit server component -> reported with environment "server", depth 1.
export function ServerWidget() {
  return <Button />;
}
