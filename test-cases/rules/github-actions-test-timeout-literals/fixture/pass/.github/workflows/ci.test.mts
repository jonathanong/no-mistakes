it('keeps the step under the job budget', () => {
  expect(step?.['timeout-minutes']).toBeLessThanOrEqual(job?.['timeout-minutes'])
})

it('gates on a name, not a value', () => {
  expect(job?.['timeout-minutes']).toContain('HOST_SUPERVISION_READY')
})
