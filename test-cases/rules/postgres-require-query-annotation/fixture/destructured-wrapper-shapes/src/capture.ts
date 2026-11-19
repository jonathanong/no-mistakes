const labels: string[] = []

async function capture(label: string, work: () => Promise<unknown>, suffix?: string): Promise<void> {
  labels.push(`${label}:${suffix}`)
  await work()
}

function withCapture<Args extends unknown[]>(
  inner: (...args: Args) => Promise<void>,
): (...args: Args) => Promise<void> {
  return async (...args) => {
    try {
      await inner(...args)
    } catch (error) {
      labels.push(String(error))
    }
  }
}

export const captureAll = withCapture(capture)
