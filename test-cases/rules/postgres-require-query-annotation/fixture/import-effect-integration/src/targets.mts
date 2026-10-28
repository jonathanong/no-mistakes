export function plain(query, value) { query.append(value) }
export function defaults(query, { suffix = 'x' } = {}) { query.append(suffix) }
export function captured(query, value) { schedule(() => { query.append(value) }) }
export function returned(query, value) { return query.append(value) }
