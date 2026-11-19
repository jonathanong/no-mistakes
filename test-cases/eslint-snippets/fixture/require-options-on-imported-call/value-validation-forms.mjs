import send from '@example/transport'
import agents, { 'other-agent' as makeOther } from '@example/agents'
await send(request, { ...rest, agent: makeOther() })
await send(request, { ['agent']: agents['otherAgent']() })
await send(request, { ['agent']: agents[name]() })
await send(request, { agent: agents() })
await send(request, { agent: (0, makeOther)() })
