const callOpenAi = /* no-mistakes: integration=openai */ async () => {}

test('uses a disallowed integration', async () => {
  await callOpenAi()
})
