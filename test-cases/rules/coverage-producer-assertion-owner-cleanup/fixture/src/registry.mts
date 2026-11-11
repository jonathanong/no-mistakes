let shutdownA: (() => unknown) | undefined;
let shutdownB: (() => unknown) | undefined;
let injected: { shutdownA?: () => unknown; shutdownB?: () => unknown } = {};

export function register(first: () => unknown, second: () => unknown) {
  shutdownA = first;
  shutdownB = second;
}

export function configure(dependencies: typeof injected) {
  injected = dependencies;
}

export function gracefulShutdown() {
  const first = injected.shutdownA ?? shutdownA;
  const second = injected.shutdownB ?? shutdownB;
  return Promise.resolve().then(first).then(second);
}
