import sql from 'sql-template-strings'
// Eight append links are admitted; the ninth exceeds the finite caller budget.
const acceptedRoot = factory(sql`/* root */ SELECT `)
const acceptedBudget = factory(sql`/* bounded */ SELECT `).append(' x').append(' x').append(' x').append(' x').append(' x').append(' x').append(' x').append(' x')
const rejectedBudget = factory(sql`/* bounded */ SELECT `).append(' x').append(' x').append(' x').append(' x').append(' x').append(' x').append(' x').append(' x').append(' x')
const rejectedMember = namespace.factory(sql`/* member */ SELECT `)
const rejectedArgument = factory(opaque())
const rejectedHole = factory(sql`/* hole */ SELECT ${opaque()}`)
const rejectedTag = factory(namespace.sql`/* tag */ SELECT `)
