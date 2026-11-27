// no-mistakes-disable-file playwright-no-hoisted-unique-token -- kept for this fixture
const suffix = randomSuffix();
test.beforeAll(async () => {
  await createPost({ slug: `post-${suffix}` });
});
