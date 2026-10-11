export const projects = [
  { test: { name: 'web', include: ['web/**/*.test.mts'], setupFiles: ['./test-helpers/web.setup.mts'] } },
  { test: { name: 'other', include: ['other/**/*.test.mts'] } },
];
