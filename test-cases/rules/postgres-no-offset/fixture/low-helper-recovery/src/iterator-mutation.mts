import { it } from 'vitest'

it.each = replacement
it.each(['placement_id'])('checks a field', async field => {
  await query(`UPDATE retained_image_placement_bindings SET ${field} = $1`)
})
