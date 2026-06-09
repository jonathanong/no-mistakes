import { Button, ButtonProps } from "../components/button";

// Spreads props, so the usages query reports hasSpread with a partial prop list.
export function Dashboard(props: ButtonProps) {
  return <Button {...props} />;
}
