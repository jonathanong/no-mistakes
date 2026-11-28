use super::super::super::lex::Boundary;
use super::super::state::State;
use super::Flow;
use crate::fx::{FxHashMap, FxHashSet};

impl Default for Flow<'_> {
    fn default() -> Self {
        Self {
            state: State::default(),
            unanalyzable: Vec::new(),
            frames: Vec::new(),
            incoming: Boundary::End,
            pending: None,
            pending_cond: None,
            seen_functions: FxHashMap::default(),
            redefined_functions: FxHashSet::default(),
            fn_exits: FxHashSet::default(),
            fn_nonempty: FxHashMap::default(),
            fn_nounset: FxHashMap::default(),
            fn_redirected: FxHashSet::default(),
            fn_unsets: FxHashMap::default(),
            fn_dynamic_unsets: FxHashSet::default(),
            restart: None,
            loop_nest_frame: None,
            loop_replays_left: 0,
        }
    }
}
