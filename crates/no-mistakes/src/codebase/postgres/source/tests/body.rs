use super::super::{body, locations::Locations, procedural};
use super::fixture;
use sqlparser::tokenizer::Token;

#[test]
fn decoded_body_boundaries_reject_foreign_source_tokens() {
    for name in ["conditional.sql", "conditional-single-quoted.sql"] {
        let sql = fixture(name);
        let prepared = crate::codebase::postgres::parse::prepare_postgres_tokens(&sql);
        let token = prepared
            .tokens
            .iter()
            .find(|token| {
                matches!(
                    token.token,
                    Token::SingleQuotedString(_) | Token::DollarQuotedString(_)
                )
            })
            .unwrap();
        let locations = Locations::new(&sql);
        let span = locations.span(token.span).unwrap();
        let program = body::decode(&token.token, &span, &sql).unwrap();
        assert_eq!(program.offset(0), Some(program.start));
        assert_eq!(program.offset(program.sql.len()), Some(program.end));
        assert_eq!(program.offset(program.sql.len() + 1), None);
        let empty = fixture("empty.sql");
        assert!(body::decode(&token.token, &Locations::new(&empty).range(0, 0), &sql).is_err());
        assert!(body::decode(&token.token, &span, &empty).is_err());
        assert!(body::decode(&prepared.tokens[0].token, &span, &sql).is_err());
        if name == "conditional-single-quoted.sql" {
            // Parent-program tokens must not masquerade as tokens of its decoded child.
            let mut foreign = token.clone();
            assert!(procedural::relocate(&mut foreign, &locations, &locations, &program).is_err());
            let mut delimiter = prepared
                .tokens
                .iter()
                .rfind(|token| token.token == Token::SemiColon)
                .unwrap()
                .clone();
            assert!(
                procedural::relocate(&mut delimiter, &locations, &locations, &program).is_err()
            );
        }
    }
}
