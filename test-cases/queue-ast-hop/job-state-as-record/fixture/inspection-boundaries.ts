import { repairs, tileQueue } from './barrel.js'
import { lookalike } from './lookalike.js'

type TileRow = { id: string }
declare function pendingTiles(limit: number): Promise<TileRow[]>

export async function restoreFromDatabase() {
  const pending = pendingTiles(100)
  const rows = await pending
  const jobs = rows.map(({ id }) => ({ name: 'restore', data: { id } }))
  await repairs.addBulk(jobs)
}

export async function inspectRequestedStates(states: string[]) {
  for (const job of await tileQueue.getJobs(states)) await job.retry()
}

export async function pruneCompletedTiles() {
  for (const job of await tileQueue.getCompleted(0, 100)) {
    if (await job.isCompleted()) {
      const labels = job.data.labels.map((label: string) => label.trim())
      console.log(job.name, labels)
      await job.remove()
    }
  }
}

export async function inspectUsingAdapter(id: string, adapter: { queue: typeof lookalike }) {
  const job = await tileQueue.getJob(id)
  if ((await job.getState()) === 'failed') {
    await adapter.queue.add('restore', { id })
    await (await Promise.resolve(adapter)).queue.add('restore', { id })
  }
  return await job?.data
}

export async function prepareCompletedTile(id: string) {
  const job = await tileQueue.getJob(id)
  if (job == null) return
  const status = await job.getState()
  if (status) console.log(status)
  const view = { payload: job.data }
  console.log(view)
}

export async function refreshPeriodicJobs() {
  let job
  for (job of await tileQueue.getFailed(0, 100)) console.log(job.name)
  const { length } = await tileQueue.getCompleted(0, 100)
  console.log(length)
}
