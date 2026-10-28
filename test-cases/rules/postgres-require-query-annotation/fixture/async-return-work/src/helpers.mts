// Many calls and aliases exercise canonical reference work without executing fixtures.
import sql from 'sql-template-strings'
import other from 'custom-sql'
async function native() {
  const nativeQuery = sql`/* work */ SELECT 1`
  return nativeQuery
}
export { native as native0 }
export { native as native1 }
export { native as native2 }
export { native as native3 }
export { native as native4 }
export { native as native5 }
export { native as native6 }
export { native as native7 }
native()
native()
native()
native()
native()
native()
native()
native()
native()
native()
native()
native()
native()
native()
native()
native()
async function custom() {
  const customQuery = other`/* work */ SELECT 1`
  return customQuery
}
export { custom as custom0 }
export { custom as custom1 }
export { custom as custom2 }
export { custom as custom3 }
export { custom as custom4 }
export { custom as custom5 }
export { custom as custom6 }
export { custom as custom7 }
custom()
custom()
custom()
custom()
custom()
custom()
custom()
custom()
custom()
custom()
custom()
custom()
custom()
custom()
custom()
custom()
