import { Card } from "../ui/Card";
import { ServerWidget } from "../ui/ServerWidget";

// Imports two components that both import Button, so Button's reverse traversal
// reaches this file twice (exercises the already-visited guard).
export default function Combo() {
  return (
    <>
      <Card />
      <ServerWidget />
    </>
  );
}
