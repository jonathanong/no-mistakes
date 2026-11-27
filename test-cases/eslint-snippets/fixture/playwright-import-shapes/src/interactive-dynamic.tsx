import UI from "./ui";

declare const componentName: string;

const Action = UI[componentName];
export const Panel = () => <Action />;
function Local() {
  return null;
}
export const Quiet = () => <Local />;
