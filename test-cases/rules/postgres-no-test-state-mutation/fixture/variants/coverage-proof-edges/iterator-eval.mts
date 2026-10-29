import { it } from 'vitest'

it.each(['appears finite'])('rejected because eval may mutate the runner', field => {
  eval('it.each = other')
  query(field)
})
