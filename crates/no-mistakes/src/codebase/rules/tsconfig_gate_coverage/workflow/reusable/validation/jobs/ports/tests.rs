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
