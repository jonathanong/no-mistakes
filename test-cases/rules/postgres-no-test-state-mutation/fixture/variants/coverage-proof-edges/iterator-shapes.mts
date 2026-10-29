import { it } from 'vitest'

it('a direct runner call', () => {})

const rows = ['not inline']
it.each(rows)('rejected', field => { query(field) })
it.each([1])('rejected', field => { query(field) })
it.each(['escaped\nrow'])('rejected', field => { query(field) })
it.each([])('rejected', field => { query(field) })
it.each(['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q'])('rejected', field => { query(field) })
it.each(['wrong callback'])('rejected', function (field) { query(field) })
it.each(['no parameter'])('rejected', () => { query('no parameter') })
it.each(['destructured'])('rejected', ({ field }) => { query(field) })
it.each(['defaulted'])('rejected', (field = 'other') => { query(field) })
it.each(['rest'])('rejected', (...field) => { query(field[0]) })
it.each(['extra argument'])('rejected', field => {}, 1)
it.each(['wrong inner argument'], 1)('rejected', field => { query(field) })
it.each?.(['optional inner'])('rejected', field => { query(field) })
it.each(['optional outer'])?.('rejected', field => { query(field) })
it['each'](['computed member'])('rejected', field => { query(field) })
(it as any).each(['wrapped runner'])('rejected', field => { query(field) })
it.each(['outer not called'])
