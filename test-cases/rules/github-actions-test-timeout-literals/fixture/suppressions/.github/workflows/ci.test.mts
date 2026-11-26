it('asserts the selected job', () => {
  expect(job?.['timeout-minutes']).toContain("&& '45'") // no-mistakes-disable-line github-actions-test-timeout-literals
})

it('asserts the sibling job', () => {
  expect(siblingJob?.['timeout-minutes']).toContain("&& '45'")
})
