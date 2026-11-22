import { scheduleOrders as schedulePack } from "./widgets";

schedulePack({ opts: { delay: 1 } });

export function byParameter(schedulePack: (template: object) => void) {
  schedulePack({ opts: { delay: 2 } });
}

export function byLocal() {
  const schedulePack = (template: object) => template;
  return schedulePack({ opts: { delay: 3 } });
}
