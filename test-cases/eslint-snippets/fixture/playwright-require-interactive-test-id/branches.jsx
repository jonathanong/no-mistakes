import Button from "./ui";
import { Button as Action } from "./ui";
import { Card } from "./ui";

const Alias = Action;
const Again = Action;

function Shadowed() {
  const Action = () => null;
  const Local = Action;
  return <Local />;
}

export function Form() {
  return (
    <>
      <button />
      {/* no-mistakes-disable-next-line playwright-require-interactive-test-id */}
      <button />
      <Action />
      <Alias />
      <Again />
      <Button />
      <UI.Button />
      <UI.Icons.Button />
      <a href="/next" />
      <div role="button" />
      <div onClick={() => {}} />
      <span />
      <Shadowed />
    </>
  );
}
