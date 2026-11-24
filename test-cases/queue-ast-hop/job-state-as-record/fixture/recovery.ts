import { repairs, tileQueue } from './barrel.js'

type TileJob = { name: string; data: { id: string } }

export async function restoreIncompleteTile(id: string) {
  const job = await tileQueue.getJob(id)
  if (true === await job.isCompleted()) {
    return
  } else {
    await repairs.add('restore', { id })
  }
}

export async function restoreBatch(baseJobs: TileJob[]) {
  for (const job of await tileQueue.getFailed(0, 100)) {
    const data = { ...job.data, recovered: true }
    await repairs.addBulk([...baseJobs, { name: job.name, data }])
  }
}
