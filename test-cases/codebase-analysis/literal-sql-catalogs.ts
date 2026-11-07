const CATALOG = { github: { table: 'users' } } as const
const SPREAD_CATALOG = { ...CATALOG } as const
type CatalogKey = keyof typeof CATALOG
function finite(key: CatalogKey) { return CATALOG[key] }
function optional(key: CatalogKey) { return CATALOG?.[key] }
function unknownGlobal(key: string) { return globalCatalog[key] }

const unary = row => !row.relation
const conditional = row => row.key === 'a' ? row.relation.table : 'fallback'
const addition = row => row.key + row.relation.table
const computed = row => row['relation']['table']
const numeric = row => 42
const logical = row => row.key && row.relation.table
const unaryOther = row => typeof row.key
const optionalMember = row => row?.relation
const block = row => { return row.relation.table }
const destructured = ({ key }) => key
const asynchronous = async row => row.relation.table
const rest = (...rows) => rows[0].key
const impure = row => consume(row.relation)
