use super::combine;

#[test]
fn product_above_32_stays_unproven() {
    let proofs: Vec<Vec<Vec<Vec<usize>>>> = (0..33).map(|index| vec![vec![vec![index]]]).collect();
    assert!(combine(&[Vec::new()], &proofs).is_none());
}

#[test]
fn each_label_keeps_its_source() {
    let first = combine(&[Vec::new()], &[vec![vec![vec![1]]]]).unwrap();
    let second = combine(&first, &[vec![vec![vec![2]]], vec![vec![vec![3]]]]).unwrap();
    assert_eq!(
        second,
        vec![
            vec![vec![vec![1]], vec![vec![2]]],
            vec![vec![vec![1]], vec![vec![3]]]
        ]
    );
}
