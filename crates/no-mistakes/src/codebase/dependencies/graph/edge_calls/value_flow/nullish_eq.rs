use super::*;
use std::sync::Arc;

#[derive(Clone)]
pub(super) struct NullishBranches {
    pub(super) value: Arc<FlowValue>,
    pub(super) fallback: Arc<FlowValue>,
}
impl PartialEq for NullishBranches {
    fn eq(&self, other: &Self) -> bool {
        self.compare(other).0
    }
}
impl NullishBranches {
    fn compare(&self, other: &Self) -> (bool, usize) {
        let mut steps = 0;
        let mut pending = vec![
            (self.value.as_ref(), other.value.as_ref()),
            (self.fallback.as_ref(), other.fallback.as_ref()),
        ];
        let mut seen = fx_set();
        while let Some((left, right)) = pending.pop() {
            steps += 1;
            if std::ptr::eq(left, right)
                || !seen.insert((left as *const FlowValue, right as *const FlowValue))
            {
                continue;
            }
            match (left, right) {
                (FlowValue::Nullish(left), FlowValue::Nullish(right)) => {
                    pending.push((&left.value, &right.value));
                    pending.push((&left.fallback, &right.fallback));
                }
                (
                    FlowValue::Annotated {
                        executor: left,
                        file: lf,
                        source: ls,
                    },
                    FlowValue::Annotated {
                        executor: right,
                        file: rf,
                        source: rs,
                    },
                ) => {
                    if lf != rf || ls != rs {
                        return (false, steps);
                    }
                    pending.push((left, right));
                }
                (
                    FlowValue::Provenance {
                        executor: left,
                        typed: lt,
                        members: lm,
                        receiver_factories: lf,
                    },
                    FlowValue::Provenance {
                        executor: right,
                        typed: rt,
                        members: rm,
                        receiver_factories: rf,
                    },
                ) => {
                    if lt != rt || lm != rm || lf != rf {
                        return (false, steps);
                    }
                    pending.push((left, right));
                }
                (FlowValue::Promise(left), FlowValue::Promise(right)) => {
                    pending.push((left, right))
                }
                (
                    FlowValue::Bound {
                        target: left,
                        arguments: la,
                    },
                    FlowValue::Bound {
                        target: right,
                        arguments: ra,
                    },
                ) => {
                    if la != ra {
                        return (false, steps);
                    }
                    pending.push((left, right));
                }
                (
                    FlowValue::Returned {
                        executor: left,
                        factory: lf,
                        receiver: lr,
                    },
                    FlowValue::Returned {
                        executor: right,
                        factory: rf,
                        receiver: rr,
                    },
                ) => {
                    if lf != rf || lr != rr {
                        return (false, steps);
                    }
                    pending.push((left, right));
                }
                _ if left != right => return (false, steps),
                _ => {}
            }
        }
        (true, steps)
    }
}
impl Eq for NullishBranches {}
impl FlowValue {
    pub(super) fn nullish(value: FlowValue, fallback: FlowValue) -> Self {
        Self::Nullish(NullishBranches {
            value: Arc::new(value),
            fallback: Arc::new(fallback),
        })
    }
}

#[cfg(test)]
#[path = "nullish_eq_tests.rs"]
mod tests;
