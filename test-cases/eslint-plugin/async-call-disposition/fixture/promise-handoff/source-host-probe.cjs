module.exports = {
  rules: {
    'no-program': {
      meta: { type: 'problem', schema: [] },
      create(context) {
        if (context.sourceCode.parserServices?.program) {
          throw new Error('Expected the real Oxlint JS plugin host without a TypeScript program');
        }
        return {};
      },
    },
  },
};
