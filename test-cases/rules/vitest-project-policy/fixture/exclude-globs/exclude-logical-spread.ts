const label = 'widgets';
const note = 'orders';
const title = 'items';
const base = { retry: 1 };
export default {
  test: {
    ...(label && { name: 'widgets' }),
    ...(note || { note: 'orders' }),
    ...(title ?? { title: 'items' }),
    ...(base && { name: 'items' }),
    ...(base || { exclude: ['**/widgets/**'] }),
    ...(base ?? { exclude: ['**/orders/**'] }),
    ...(null && { name: 'skipped' }),
    ...(null ?? { hookTimeout: 5 }),
    ...((label && { name: 'widgets' }) && { retry: 1 }),
    ...((label && { note: 'orders' }) || { title: 'items' }),
    ...((label && { name: 'widgets' }) ?? { hookTimeout: 5 }),
    projects: [{ test: { name: 'items', isolate: true } }],
  },
};
