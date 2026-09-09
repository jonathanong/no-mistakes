import { setTimeout as wait } from "node:timers";

wait(() => {}, 1);

export function unknownCall(
  runner: Record<string, () => void>,
  name: string,
) {
  runner[name]();
}
