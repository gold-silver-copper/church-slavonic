use church_slavonic_tagger::Tagger;

fn model(records: &[(u64, f32)]) -> Vec<u8> {
    let mut bytes = b"CST1".to_vec();
    bytes.extend_from_slice(&(records.len() as u32).to_le_bytes());
    for (key, weight) in records {
        bytes.extend_from_slice(&key.to_le_bytes());
        bytes.extend_from_slice(&weight.to_le_bytes());
    }
    bytes
}

#[test]
fn validates_binary_payload_before_allocating_or_accepting_weights() {
    assert!(Tagger::from_bytes(&model(&[])).is_some());
    assert!(Tagger::from_bytes(&model(&[(1, 1.0)])).is_some());
    for weight in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(Tagger::from_bytes(&model(&[(1, weight)])).is_none());
    }
    assert!(Tagger::from_bytes(&model(&[(1, 1.0), (1, 2.0)])).is_none());
    let mut trailing = model(&[(1, 1.0)]);
    trailing.push(0);
    assert!(Tagger::from_bytes(&trailing).is_none());
    let valid = model(&[(1, 1.0)]);
    for end in 0..valid.len() {
        assert!(Tagger::from_bytes(&valid[..end]).is_none());
    }
    let mut huge_header = b"CST1".to_vec();
    huge_header.extend_from_slice(&u32::MAX.to_le_bytes());
    assert!(Tagger::from_bytes(&huge_header).is_none());
}
