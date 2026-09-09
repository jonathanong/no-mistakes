export function boot() {
  const registry = {
    load: () => import('./object-called.mts'),
    unused: () => import('./object-unused.mts'),
  };
  registry.load();

  class Service {
    static run = () => import('./field-called.mts');
    static reload = function () {
      import('./field-reloaded.mts');
    };
    static unused = () => import('./field-unused.mts');
  }
  Service.run();
  Service.reload();
}

boot();
