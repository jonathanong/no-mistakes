use super::Projector;
use oxc_ast::ast::*;

enum Supplied<'b, 'a> {
    Value(&'b Expression<'a>),
    Missing,
    Unknown,
}

impl<'a> Projector<'a, '_> {
    pub(super) fn definitely_nullish(&mut self, expression: &Expression<'a>, depth: usize) -> bool {
        if depth >= 16 || !self.take() {
            return false;
        }
        match expression.get_inner_expression() {
            Expression::NullLiteral(_) => true,
            Expression::Identifier(id) => {
                if id.name == "undefined" && self.binding_symbol(id).is_none() {
                    return true;
                }
                let Some(init) = self.const_init(id) else {
                    return false;
                };
                self.definitely_nullish(init, depth + 1)
            }
            Expression::StaticMemberExpression(member) => {
                match self.nullish_property(
                    &member.object,
                    member.property.name.as_str(),
                    depth + 1,
                ) {
                    Supplied::Missing => true,
                    Supplied::Value(value) => self.definitely_nullish(value, depth + 1),
                    Supplied::Unknown => false,
                }
            }
            Expression::ChainExpression(chain) => {
                let ChainElement::StaticMemberExpression(member) = &chain.expression else {
                    return false;
                };
                if member.optional && self.definitely_nullish(&member.object, depth + 1) {
                    return true;
                }
                match self.nullish_property(
                    &member.object,
                    member.property.name.as_str(),
                    depth + 1,
                ) {
                    Supplied::Missing => true,
                    Supplied::Value(value) => self.definitely_nullish(value, depth + 1),
                    Supplied::Unknown => false,
                }
            }
            _ => false,
        }
    }

    fn nullish_property<'b>(
        &mut self,
        object: &'b Expression<'a>,
        key: &str,
        depth: usize,
    ) -> Supplied<'b, 'a> {
        if depth >= 16 || !self.take() {
            return Supplied::Unknown;
        }
        match object.get_inner_expression() {
            Expression::Identifier(id) => {
                let Some(symbol) = self.binding_symbol(id) else {
                    return Supplied::Unknown;
                };
                if !self.member_reads_only(symbol) {
                    return Supplied::Unknown;
                }
                let Some(init) = self.const_init(id) else {
                    return Supplied::Unknown;
                };
                self.nullish_property(init, key, depth + 1)
            }
            Expression::ObjectExpression(object) => {
                if object.properties.len() > 32 {
                    return Supplied::Unknown;
                }
                let mut supplied = Supplied::Missing;
                for property in &object.properties {
                    match property {
                        ObjectPropertyKind::SpreadProperty(spread) => {
                            let next = self.nullish_property(&spread.argument, key, depth + 1);
                            if !matches!(next, Supplied::Missing) {
                                supplied = next;
                            }
                        }
                        ObjectPropertyKind::ObjectProperty(property) => {
                            if property.computed {
                                supplied = Supplied::Unknown;
                                continue;
                            }
                            if crate::ast::property_key_name(&property.key).as_deref() == Some(key)
                            {
                                supplied = if property.method || property.kind != PropertyKind::Init
                                {
                                    Supplied::Unknown
                                } else {
                                    Supplied::Value(&property.value)
                                };
                            }
                        }
                    }
                }
                supplied
            }
            Expression::StaticMemberExpression(member) => {
                match self.nullish_property(
                    &member.object,
                    member.property.name.as_str(),
                    depth + 1,
                ) {
                    Supplied::Value(value) => self.nullish_property(value, key, depth + 1),
                    _ => Supplied::Unknown,
                }
            }
            _ => Supplied::Unknown,
        }
    }
}
