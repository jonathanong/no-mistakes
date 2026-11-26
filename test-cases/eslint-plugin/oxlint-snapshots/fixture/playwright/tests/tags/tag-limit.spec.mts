const suffix = randomSuffix();
test.beforeAll(async () => {
  await createTag({ slug: `tag-${suffix}` });
});
