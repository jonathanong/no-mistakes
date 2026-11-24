import { repairs, tileQueue } from './barrel.js'
import { lookalike } from './lookalike.js'

declare function pendingTiles(limit: number): Promise<{ id: string }[]>

export async function retryFromDurableRecords() {
  for (const row of await pendingTiles(100)) {
    await repairs.add('restore', { id: row.id })
  }
}

export async function inspect(id: string) {
  const job = await tileQueue.getJob(id)
  if (await job.isFailed()) return job.data
  return await job.getState()
}

export async function differentJob(firstId: string, secondId: string) {
  const first = await tileQueue.getJob(firstId)
  const second = await tileQueue.getJob(secondId)
  if (await first.isFailed()) await second.retry()
}

export async function delayedCallback(id: string) {
  const job = await tileQueue.getJob(id)
  if (await job.isFailed()) return () => job.retry()
}

export async function nonterminalJobs() {
  for (const job of await tileQueue.getJobs(['waiting'])) await job.retry()
}

export async function otherBindings(id: string) {
  const job = await lookalike.getJob(id)
  if (await job.isFailed()) await job.retry()
  const realJob = await tileQueue.getJob(id)
  if (await realJob.isFailed()) await lookalike.add('restore', { id })
}

export async function shadowedQueue(tileQueue: typeof lookalike, id: string) {
  const job = await tileQueue.getJob(id)
  if (await job.isFailed()) await job.retry()
}

export async function unrelatedWork() {
  for (const job of await tileQueue.getFailed(0, 100)) {
    console.log(job.name)
    await repairs.add('heartbeat', {})
  }
}
