async function getterPromiseAfterBarrier() {
  const p = Promise.resolve("outer");
  with (
    {
      get p() {
        return work();
      },
    }
  ) {
    await barrier();
    const alias = p;
    await expect(alias).rejects.toThrow();
  }
}

async function getterInterceptsAliasAtMatcher() {
  const p = Promise.resolve("outer");
  const alias = p;
  with (
    {
      get alias() {
        return work();
      },
    }
  ) {
    await barrier();
    await expect(alias).rejects.toThrow();
  }
}

async function lexicalAliasesInsideWith() {
  with ({ unrelated: true }) {
    const p = work();
    const alias = p;
    await barrier();
    await expect(alias).rejects.toThrow();
  }
}
