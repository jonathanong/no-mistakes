use super::super::PathDefault;
use super::{PathAnchor, PathOrigin, Projector};
use oxc_ast::ast::*;

impl PathOrigin {
    pub(crate) fn atoms(&self) -> &[Self] {
        if self.alternatives.is_empty() {
            std::slice::from_ref(self)
        } else {
            &self.alternatives
        }
    }

    pub(crate) fn alternatives(origins: Vec<Self>) -> Self {
        let mut alternatives = Vec::new();
        for origin in origins {
            for atom in origin.atoms() {
                if !alternatives.contains(atom) {
                    alternatives.push(atom.clone());
                }
                if alternatives.len() > 16 {
                    return Self::unknown();
                }
            }
        }
        if alternatives.len() == 1 {
            return alternatives.pop().unwrap();
        }
        Self {
            alternatives,
            ..Self::unknown()
        }
    }

    pub(super) fn coalesced(mut self, right: Self) -> Self {
        if self.nullish {
            return right;
        }
        if self.param.is_none() || self.param_member.is_none() {
            return Self::alternatives(vec![self, right]);
        }
        match self.param_default.take() {
            Some(default) if default.nullish => {
                self.param_default = Some(PathDefault {
                    value: Box::new(default.value.coalesced(right)),
                    nullish: true,
                });
            }
            Some(default) => {
                self.param_default = Some(default);
                return Self::alternatives(vec![self, right]);
            }
            None => {
                self.param_default = Some(PathDefault {
                    value: Box::new(right),
                    nullish: true,
                })
            }
        }
        self
    }

    pub(super) fn resolve_append(self, tail: Self) -> Self {
        if self.alternatives.is_empty() && tail.alternatives.is_empty() {
            return if tail.anchor == PathAnchor::Relative {
                self.append(tail)
            } else if tail.param.is_some() && tail.param_member.is_some() {
                Self::alternatives(vec![
                    self,
                    Self {
                        relative: true,
                        ..tail
                    },
                ])
            } else {
                tail
            };
        }
        let alternatives = self
            .atoms()
            .iter()
            .flat_map(|base| {
                tail.atoms().iter().map(|tail| {
                    if tail.anchor == PathAnchor::Relative {
                        base.clone().append(tail.clone())
                    } else if tail.param.is_some() && tail.param_member.is_some() {
                        Self::alternatives(vec![
                            base.clone(),
                            Self {
                                relative: true,
                                ..tail.clone()
                            },
                        ])
                    } else {
                        tail.clone()
                    }
                })
            })
            .collect();
        Self::alternatives(alternatives)
    }
}

impl<'a> Projector<'a, '_> {
    pub(super) fn chain_origin(&mut self, chain: &ChainExpression<'a>, depth: usize) -> PathOrigin {
        match &chain.expression {
            ChainElement::StaticMemberExpression(member) => self
                .member_origin(&member.object, member.property.name.as_str(), depth + 1)
                .unwrap_or_else(PathOrigin::unknown),
            _ => PathOrigin::unknown(),
        }
    }

    pub(in super::super) fn member_initializer<'b>(
        &mut self,
        object: &'b Expression<'a>,
        key: &str,
        depth: usize,
    ) -> Option<&'b Expression<'a>> {
        if depth >= 16 || !self.take() {
            return None;
        }
        match object.get_inner_expression() {
            Expression::Identifier(id) => {
                let symbol = self.binding_symbol(id)?;
                if !self.member_reads_only(symbol) {
                    return None;
                }
                let init = self.const_init(id)?;
                self.member_initializer(init, key, depth + 1)
            }
            Expression::ObjectExpression(object) => {
                let mut value = None;
                for property in &object.properties {
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        return None;
                    };
                    if property.computed || property.method || property.kind != PropertyKind::Init {
                        return None;
                    }
                    if crate::ast::property_key_name(&property.key).as_deref() == Some(key) {
                        value = Some(&property.value);
                    }
                }
                value
            }
            Expression::StaticMemberExpression(member) => {
                let value = self.member_initializer(
                    &member.object,
                    member.property.name.as_str(),
                    depth + 1,
                )?;
                self.member_initializer(value, key, depth + 1)
            }
            _ => None,
        }
    }
}
