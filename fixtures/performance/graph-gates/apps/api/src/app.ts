const app = {
  route(path: string) { return this; },
  get(path: string, handler?: unknown) { return this; },
  post(path: string, handler?: unknown) { return this; },
  patch(path: string, handler?: unknown) { return this; },
  delete(path: string, handler?: unknown) { return this; },
};
export default app;
