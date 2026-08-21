it('outside workflows is ignored even with a literal', () => {
  expect(step?.['timeout-minutes']).toBe(10)
})
