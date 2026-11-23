export function createCustomConfig() {
  return {
    test: {
      exclude: ['custom/**'],
    },
  };
}

export function createSuppressedConfig() {
  return {
    test: {
      // no-mistakes-disable-next-line vitest-default-excludes
      exclude: ['suppressed/**'],
    },
  };
}

export function createCoverageOnlyConfig() {
  return {
    test: {
      coverage: { exclude: ['custom/**'] },
    },
  };
}
