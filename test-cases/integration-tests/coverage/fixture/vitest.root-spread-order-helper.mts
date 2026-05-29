export default {
  include: ['root-spread-order/**/*.test.ts'],
  projects: [
    {
      test: {
        name: 'root-spread-order-shared',
      },
    },
  ],
}
