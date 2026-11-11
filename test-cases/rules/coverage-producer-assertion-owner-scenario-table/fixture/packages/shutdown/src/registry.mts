export function createRegistry(log: any) {
  let registeredA = async () => {};
  let registeredB = async () => {};
  return {
    registerA(callback: () => Promise<void>) { registeredA = callback; },
    registerB(callback: () => Promise<void>) { registeredB = callback; },
    async close(deps: any = {}) {
      const shutdownA = deps.onA ?? registeredA;
      const shutdownB = deps.onB ?? registeredB;
      const logger = deps.log ?? log;
      const results = await Promise.allSettled([
        Promise.resolve().then(shutdownA),
        Promise.resolve().then(shutdownB),
      ]);
      for (const result of results) {
        if (result.status === 'fulfilled') logger.info('shutdown complete');
      }
      return results.flatMap(result => result.status === 'rejected' ? [toError(result.reason)] : []);
    },
  };
}
function toError(reason: unknown) { return reason instanceof Error ? reason : new Error(String(reason)); }
