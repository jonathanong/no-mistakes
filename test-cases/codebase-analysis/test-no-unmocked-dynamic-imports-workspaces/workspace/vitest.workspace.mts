export default [
  { test: { name: 'web', include: ['custom/web.case.mts'], setupFiles: ['./setup.mts'] } },
  { test: { name: 'other', include: ['custom/other.case.mts'] } },
];
