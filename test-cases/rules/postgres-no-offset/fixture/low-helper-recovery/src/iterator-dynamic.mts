import { it } from 'vitest'

const fields = ['placement_id', 'image_id']
it.each(fields)('checks a field', async field => {
  await query(`UPDATE retained_image_placement_bindings SET ${field} = $1`)
})
