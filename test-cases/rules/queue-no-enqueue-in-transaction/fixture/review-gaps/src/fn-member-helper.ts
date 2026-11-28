import { enqueue } from '@sample/jobs'

export function helpers() {}
helpers.send = enqueue

export const arrowHelper = () => {}
arrowHelper.send = enqueue
