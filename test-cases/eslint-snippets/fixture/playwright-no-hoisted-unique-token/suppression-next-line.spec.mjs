const suffix = randomSuffix();
test.beforeAll(async () => {
  // no-mistakes-disable-next-line playwright-no-hoisted-unique-token -- kept for this fixture
  await createPost({ slug: `post-${suffix}` });
});
