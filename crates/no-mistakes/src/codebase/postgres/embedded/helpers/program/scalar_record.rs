use super::{Entry, Expr, Param, Program, Stmt};
use crate::codebase::postgres::embedded::helpers::SiteArg;
use crate::fx::FxHashSet;

pub(super) fn fields(params: &[Param], body: &[Stmt]) -> Vec<String> {
    let [Stmt::Return(Some(Expr::Object(entries)), _)] = body else {
        return Vec::new();
    };
    if params
        .iter()
        .any(|param| param.builder || param.default.is_some())
    {
        return Vec::new();
    }
    let mut names = FxHashSet::default();
    for entry in entries {
        let Entry::Prop(name, value) = entry else {
            return Vec::new();
        };
        if name == "__proto__" || !names.insert(name.clone()) || !scalar(value, params) {
            return Vec::new();
        }
    }
    let mut names: Vec<_> = names.into_iter().collect();
    names.sort();
    names
}

fn scalar(value: &Expr, params: &[Param]) -> bool {
    match value {
        Expr::Lit(_) | Expr::Bool(_) | Expr::Null | Expr::Undefined | Expr::Num => true,
        Expr::Ident(name) => params.iter().any(|param| param.name == *name),
        Expr::Cond(test, yes, no) => {
            scalar(test, params) && scalar(yes, params) && scalar(no, params)
        }
        Expr::Eq(left, right, strict, _) => {
            *strict && scalar(left, params) && scalar(right, params)
        }
        _ => false,
    }
}

impl Program {
    pub(crate) fn scalar_member(&self, name: &str, args: &[SiteArg]) -> bool {
        self.scalar_fields.iter().any(|field| field == name)
            && self.params.iter().enumerate().all(|(index, _)| {
                matches!(
                    args.get(index),
                    Some(
                        SiteArg::Scalar
                            | SiteArg::Literal(_)
                            | SiteArg::Bool(_)
                            | SiteArg::Null
                            | SiteArg::Undefined
                    )
                )
            })
    }
}
