import { selector } from "./selectors";

export function App() {
  return <button data-testid={selector}>Save</button>;
}
