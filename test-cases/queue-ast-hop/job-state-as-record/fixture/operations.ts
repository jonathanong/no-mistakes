import { tileQueue } from './barrel.js'

export async function retryFromOperatorDashboard(id: string) {
  const job = await tileQueue.getJob(id)
  if (await job.isFailed()) await job.retry()
}
