export function contextExtensionRunner(hooks, maximum = 25) {
  return {
    config: { testTimeout: maximum, hookTimeout: maximum },
    extendTaskContext(context) {
      for (const [name, method] of Object.entries(hooks)) context[name] = method;
      return context;
    },
  };
}
