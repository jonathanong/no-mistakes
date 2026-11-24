import * as queues from './barrel.js'
import { repairs, tileQueue } from './barrel.js'
import { buildQueue } from './queue-factory.js'

export async function retryTile(id: string) {
  const job = await tileQueue.getJob(id)
  if ((await job?.getState()) === 'failed') await job.retry()
}

export async function removeTile(id: string) {
  const localQueue = buildQueue('local-tiles')
  const job = await localQueue.getJob(id)
  const completed = await job.isCompleted()
  if (!completed) return
  const sameJob = job
  await sameJob.remove()
}

export async function recreateTile(id: string) {
  const job = await queues.tileQueue.getJob(id)
  if (await job.isFailed()) await repairs.add('restore', { id })
}

export async function recreateFailedTiles() {
  for (const job of await tileQueue.getFailed(0, 100)) {
    await tileQueue.add(job.name, job.data)
  }
}

export async function retryFailedTiles() {
  const failed = await tileQueue.getJobs(['waiting', 'failed'])
  for (const row of failed) await row.retry()
}

export async function recreateCompletedTiles() {
  const completed = await tileQueue.getCompleted(0, 100)
  const jobs = completed.map((job: { name: string; data: unknown }) => ({
    name: job.name,
    data: job.data,
  }))
  await repairs.addBulk(jobs)
}

export async function intentionalRepair(id: string) {
  const job = await tileQueue.getJob(id)
  // no-mistakes-disable-next-line queues-check
  if (await job?.isCompleted()) await job.retry()
}
