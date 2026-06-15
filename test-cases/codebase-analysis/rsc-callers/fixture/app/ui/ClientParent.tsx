import { ClientThing } from "./ClientThing";

// Above a client boundary; must never be reported as an RSC caller of Button.
export function ClientParent() {
  return <ClientThing />;
}
