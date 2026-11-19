import { send as dispatch } from '@example/transport'
import { approvedAgent as makeApproved, otherAgent as makeOther } from '@example/agents'
await dispatch(request, { agent: makeOther() })
await dispatch(request, { agent: makeApproved() })
await dispatch(request, { agent }, extra)
await dispatch(request, { agent })
function shadowed(makeApproved) {
  return dispatch(request, { agent: makeApproved() })
}
