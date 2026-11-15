use super::{apply, Credit};

fn value_column() -> Vec<Vec<usize>> {
    vec![vec![]]
}

fn free_column() -> Vec<Vec<usize>> {
    vec![]
}

fn dependent_column() -> Vec<Vec<usize>> {
    vec![vec![0]]
}

#[test]
fn value_and_rejected_arms_add_nothing_until_a_domain_is_used() {
    let mut prepared = vec![
        vec![vec![value_column()]],
        vec![vec![vec![]]],
        vec![vec![free_column()]],
    ];
    let domains = vec![vec![vec![None]], vec![vec![None]], vec![vec![Some(4)]]];
    let credit = apply(&mut prepared, &domains, &[1], 10);
    assert_eq!(credit, Some(Credit::Accept(5)));
    assert_eq!(prepared[2][0][0], vec![Vec::<usize>::new()]);
}

#[test]
fn a_dependent_pin_is_not_a_finite_domain() {
    let mut prepared = vec![vec![vec![dependent_column()]]];
    let domains = vec![vec![vec![Some(4)]]];
    assert_eq!(apply(&mut prepared, &domains, &[], 10), None);
}

#[test]
fn products_that_overflow_do_not_accept() {
    let mut wide = vec![vec![vec![free_column(), free_column()]]];
    let domains = vec![vec![vec![Some(u64::MAX), Some(2)]]];
    assert_eq!(apply(&mut wide, &domains, &[], u64::MAX), None);

    let mut arms = vec![vec![vec![free_column()]], vec![vec![free_column()]]];
    let domains = vec![vec![vec![Some(u64::MAX)]], vec![vec![Some(u64::MAX)]]];
    assert_eq!(apply(&mut arms, &domains, &[], u64::MAX), None);
}
