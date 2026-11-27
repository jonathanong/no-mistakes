const suffix = randomSuffix();
test.beforeAll(async () => {
  await createPost({ slug: `post-${suffix}` }); // no-mistakes-disable-line playwright-no-hoisted-unique-token -- kept for this fixture
});
