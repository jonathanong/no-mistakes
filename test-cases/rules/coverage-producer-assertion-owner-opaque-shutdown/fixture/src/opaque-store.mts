export let shutdownStore: (() => unknown) | undefined;

export function registerShutdown(callback: () => unknown) {
  shutdownStore = callback;
}
