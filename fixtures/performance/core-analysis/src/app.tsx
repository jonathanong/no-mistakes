import { formatCore, type CoreValue } from "@core/index";
import { Button } from "./button";

export function App({ value }: { value: CoreValue }) {
  return <Button label={formatCore(value)} />;
}
