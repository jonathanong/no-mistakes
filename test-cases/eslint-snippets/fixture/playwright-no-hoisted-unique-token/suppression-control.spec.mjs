const suffix = randomSuffix();
test.beforeAll(async () => {
  await createPost({ slug: `post-${suffix}` });
});
