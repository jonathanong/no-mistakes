import { read } from '@bench/db'
import sql from 'sql-template-strings'

function evalCaptured() {
  const statement = sql`/* evalCaptured */ SELECT 1`
  handOff(() => { eval('statement.text = "SELECT 1"'); statement.append(dynamicTail) })
  return statement
}
function argumentsCaptured() {
  const statement = sql`/* argumentsCaptured */ SELECT 1`
  handOff(function () { arguments[0]; statement.append(dynamicTail) })
  return statement
}
read(evalCaptured()) // low: false
read(argumentsCaptured()) // low: false
