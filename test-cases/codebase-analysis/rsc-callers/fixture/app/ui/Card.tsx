import { Button } from "./Button";

// Server component (no directive) -> reported, depth 1.
export function Card() {
  return (
    <div>
      <Button />
    </div>
  );
}
