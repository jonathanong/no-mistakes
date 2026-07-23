const flag = process.env.CI === 'true'

// Deliberately exceeds the static setup branch-depth limit. Keep this as a
// literal chain so the parser's depth accounting cannot be bypassed by aliases.
const deep = flag ? './setup-01.ts'
  : flag ? './setup-02.ts'
  : flag ? './setup-03.ts'
  : flag ? './setup-04.ts'
  : flag ? './setup-05.ts'
  : flag ? './setup-06.ts'
  : flag ? './setup-07.ts'
  : flag ? './setup-08.ts'
  : flag ? './setup-09.ts'
  : flag ? './setup-10.ts'
  : flag ? './setup-11.ts'
  : flag ? './setup-12.ts'
  : flag ? './setup-13.ts'
  : flag ? './setup-14.ts'
  : flag ? './setup-15.ts'
  : flag ? './setup-16.ts'
  : flag ? './setup-17.ts'
  : flag ? './setup-18.ts'
  : flag ? './setup-19.ts'
  : flag ? './setup-20.ts'
  : flag ? './setup-21.ts'
  : flag ? './setup-22.ts'
  : flag ? './setup-23.ts'
  : flag ? './setup-24.ts'
  : flag ? './setup-25.ts'
  : flag ? './setup-26.ts'
  : flag ? './setup-27.ts'
  : flag ? './setup-28.ts'
  : flag ? './setup-29.ts'
  : flag ? './setup-30.ts'
  : flag ? './setup-31.ts'
  : flag ? './setup-32.ts'
  : flag ? './setup-33.ts'
  : './setup-34.ts'

export default {
  test: {
    projects: [
      {
        test: {
          name: 'deep-consequent',
          include: ['consequent/**/*.test.ts'],
          setupFiles: flag ? deep : './fallback.ts',
        },
      },
      {
        test: {
          name: 'deep-alternate',
          include: ['alternate/**/*.test.ts'],
          setupFiles: flag ? './fallback.ts' : deep,
        },
      },
      {
        test: {
          name: 'deep-spread',
          include: ['spread/**/*.test.ts'],
          setupFiles: [...deep],
        },
      },
    ],
  },
}
