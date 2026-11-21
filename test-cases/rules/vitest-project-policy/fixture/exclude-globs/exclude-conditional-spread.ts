const label = 'widgets';
const note = 'orders';
const base = { retry: 1 };
export default {
  test: {
    ...(label ? { name: 'widgets' } : {}),
    ...(note ? (label ? { hookTimeout: 1 } : {}) : { hookTimeout: 2 }),
    ...(label
      ? { meta: { ...(note ? { name: 'widgets' } : { name: 'orders' }) } }
      : { meta: { ...(note ? { name: 'widgets' } : { name: 'orders' }) } }),
    ...(label
      ? { alt: { ...(note ? { name: 'widgets' } : { name: 'items' }) } }
      : { alt: { ...(note ? { name: 'orders' } : { name: 'items' }) } }),
    ...(label ? { retry: base } : { retry: base }),
    ...(label ? { title: widgetSettings } : { title: widgetSettings }),
    ...(label ? { note: 'orders' } : { note: widgetSettings }),
    projects: [
      {
        test: {
          name: 'items',
          ...(label ? { isolate: true } : { isolate: true }),
        },
      },
    ],
  },
};
