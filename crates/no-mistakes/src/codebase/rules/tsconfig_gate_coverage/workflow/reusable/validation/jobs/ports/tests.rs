use super::*;

#[test]
fn ports_allow_documented_numbers_and_mappings() {
    for value in [
        "80",
        "8080:80",
        "127.0.0.1:8080:80/tcp",
        "[::1]:8080:80/udp",
        "8080-8082:80-82",
        "${{ matrix.host_port }}:80",
        "${{ matrix.host_ip }}:8080:80",
        "8080:80/${{ inputs.protocol }}",
        "${{ matrix.host_start }}-${{ matrix.host_end }}:8000-8002",
        "8080-${{ matrix.host_end }}:${{ matrix.container_start }}-${{ matrix.container_end }}",
    ] {
        assert!(port_mapping_valid(value), "{value}");
    }
    for value in [
        "0",
        "65536",
        "8080:80/icmp",
        "8080-8082:80-81",
        "not-a-port",
        "${{ matrix.host_ip }}:not-a-port:80",
        "8080:80/${{ }}",
        "${{ matrix.host_port }}-:80",
        "?",
        "?:80",
        "8080:?",
        "80/?",
    ] {
        assert!(!port_mapping_valid(value), "{value}");
    }
}

#[test]
fn ports_reject_non_port_yaml_values() {
    for yaml in ["true", "null", "{}", "[]", "0", "65536", "1.5"] {
        let value = serde_yaml::from_str::<Value>(yaml).unwrap();
        assert!(!port_entry_valid(&value), "{yaml}");
    }
}

#[test]
fn port_sequences_cover_dynamic_parts_and_invalid_mapping_shapes() {
    assert!(port_sequence_valid(None));
    assert!(port_sequence_valid(Some(
        &serde_yaml::from_str("[]").unwrap()
    )));
    assert!(!port_sequence_valid(Some(
        &serde_yaml::from_str("[80, invalid]").unwrap()
    )));

    for value in [
        "${{ matrix.port }}",
        "${{ matrix.host }}:8080:80",
        "[${{ matrix.address }}]:8080:80/sctp",
        "8080-${{ matrix.end }}:80-${{ matrix.container_end }}",
    ] {
        assert!(port_mapping_valid(value), "{value}");
    }
    for value in [
        "80/tcp/udp",
        "[::1:8080:80",
        "[not-an-address]:8080:80",
        "8080:80:81:82",
        "8080-8082:80-81",
        "${{ matrix.port }",
        "8080:${{ matrix.port }}:80",
    ] {
        assert!(!port_mapping_valid(value), "{value}");
    }

    assert_eq!(
        opaque_expression_form("${{ matrix.host }}:8080:${{ matrix.container }}"),
        format!("{DYNAMIC_EXPRESSION}:8080:{DYNAMIC_EXPRESSION}")
    );
}
