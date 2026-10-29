import { it } from 'vitest'

it.each(['placement_id', 'image_id', 'binding_family'] as const)(
  'checks the selected field',
  async field => {
    const update = { placement_id: 'placement_id = $2', image_id: 'image_id = $2', binding_family: 'binding_family = $2' }[field]
    await query(`UPDATE retained_image_placement_bindings SET ${update} WHERE placement_id = $1`)
  },
)
