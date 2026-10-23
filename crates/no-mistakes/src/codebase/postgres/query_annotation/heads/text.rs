use super::HeadProof;
use oxc_ast::ast::Expression;

/// A literal's EOF proof can change after a builder append: BEGIN may become
/// BEGINNING, and B may become BEGIN. Retain the continuation proof separately.
#[derive(Clone, Copy)]
pub(super) struct TextHead {
    pub tagged: bool,
    pub appended: bool,
    pub current: HeadProof,
    pub continued: HeadProof,
}

pub(super) fn head<'a>(
    expression: &'a Expression<'a>,
    continued: bool,
    appended: bool,
) -> Option<TextHead> {
    let (text, tagged, interpolated) = match expression {
        Expression::StringLiteral(value) => (value.value.as_str(), false, false),
        Expression::TemplateLiteral(value) => (
            value.quasis.first()?.value.cooked.as_ref()?.as_str(),
            false,
            !value.expressions.is_empty(),
        ),
        Expression::TaggedTemplateExpression(value) => (
            value.quasi.quasis.first()?.value.cooked.as_ref()?.as_str(),
            true,
            !value.quasi.expressions.is_empty(),
        ),
        _ => return None,
    };
    let (complete, continued_proof) = classify(text);
    Some(TextHead {
        tagged: tagged || appended,
        appended,
        current: if continued || interpolated || appended {
            continued_proof
        } else {
            complete
        },
        continued: continued_proof,
    })
}

fn classify(text: &str) -> (HeadProof, HeadProof) {
    // Share max's Unicode and multiple-leading-comment transaction semantics.
    let transaction = crate::codebase::postgres::annotation::transaction_command_prefix_end(text);
    let complete = if transaction.is_some() {
        HeadProof::Literal
    } else {
        HeadProof::Missing
    };
    if transaction.is_some_and(|end| end < text.len()) {
        return (complete, HeadProof::Literal);
    }
    let mut leading = text.trim_start();
    loop {
        if leading.is_empty() || leading == "/" {
            return (complete, HeadProof::Unproven);
        }
        let Some(comment) = leading.strip_prefix("/*") else {
            break;
        };
        let Some(end) = comment.find("*/") else {
            return (complete, HeadProof::Unproven);
        };
        leading = comment[end + 2..].trim_start();
    }
    if ["BEGIN", "COMMIT", "ROLLBACK"].iter().any(|command| {
        leading.len() <= command.len()
            && command
                .get(..leading.len())
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case(leading))
    }) {
        (complete, HeadProof::Unproven)
    } else {
        (complete, HeadProof::Missing)
    }
}
