use std::path::PathBuf;

/// Declaration evidence only. Invalid numeric budgets are retained for policy consumers.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DeadlineValue {
    Known(f64),
    Unknown(DeadlineUnknownReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeadlineUnknownReason {
    Expression,
    NonFinite,
    OpaqueSpread,
    OpaqueTestObject,
    ComputedProperty,
    UnresolvedExtends,
    UnresolvedInheritance,
    UnprovedConfigRoot,
    Accessor,
    UnsupportedConfigCall,
    UnprovedBinding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeadlineInheritance {
    pub(crate) path: PathBuf,
    pub(crate) project: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DeadlineDeclaration {
    pub(crate) value: DeadlineValue,
    pub(crate) path: PathBuf,
    /// None for JSON declarations, whose existing interpreter supplies no spans.
    pub(crate) span: Option<(u32, u32)>,
    pub(crate) inherited_through: Vec<DeadlineInheritance>,
}

/// None means absent, never an SDK default or a proven finite budget.
/// Fixture declarations are absent for the config forms supported in this first unit.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct DeclaredDeadlines {
    pub(crate) case: Option<DeadlineDeclaration>,
    pub(crate) hook: Option<DeadlineDeclaration>,
    pub(crate) fixture: Option<DeadlineDeclaration>,
}

impl DeclaredDeadlines {
    pub(crate) fn overlay(&mut self, next: Self) {
        if next.case.is_some() {
            self.case = next.case;
        }
        if next.hook.is_some() {
            self.hook = next.hook;
        }
        if next.fixture.is_some() {
            self.fixture = next.fixture;
        }
    }

    pub(crate) fn inherit_missing(&mut self, parent: &Self, via: DeadlineInheritance) {
        for (local, inherited) in [
            (&mut self.case, &parent.case),
            (&mut self.hook, &parent.hook),
            (&mut self.fixture, &parent.fixture),
        ] {
            if local.is_none() {
                *local = inherited.clone().map(|mut declaration| {
                    declaration.inherited_through.push(via.clone());
                    declaration
                });
            }
        }
    }
}
