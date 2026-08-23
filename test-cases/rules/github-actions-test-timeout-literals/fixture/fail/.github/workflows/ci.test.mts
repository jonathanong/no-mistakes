it('embeds a yaml fragment', () => {
  expect(workflowSource).toContain('timeout-minutes: 15')
})

it('asserts a property literal', () => {
  expect(step?.['timeout-minutes']).toBe(10)
})

it('pins a dynamic expression branch', () => {
  expect(job?.['timeout-minutes']).toContain("&& '45'")
})

it('equals a camelCase property', () => {
  expect(job.timeoutMinutes).toEqual(8)
})
