#[test]
fn saved_fixtures_flag_only_literal_host_ports() {
    let (root, config, files) = super::load_fixture("pass");
    let findings = super::super::check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "{findings:?}");

    let (root, config, files) = super::load_fixture("abstain");
    let findings = super::super::check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "{findings:?}");

    let (root, config, files) = super::load_fixture("fail");
    let findings = super::super::check_with_files(&root, &config, &files).unwrap();
    let mut got: Vec<String> = findings.iter().map(super::identity).collect();
    got.sort();
    let mut expected = Vec::new();
    for (job, spec) in [
        ("after-heredoc", "5432:5432"),
        ("attached", "5432:5432"),
        ("block", "5432:5432"),
        ("cap-add-after", "5432:5432"),
        ("cap-add-before", "5432:5432"),
        ("captured", "5432:5432"),
        ("chain", "5432:5432"),
        ("container-run", "5432:5432"),
        ("clustered", "5432:5432"),
        ("container-var", "5432:$PORT"),
        ("continued", "5432:5432"),
        ("env-prefix", "5432:5432"),
        ("hd-and", "5432:5432"),
        ("hd-pipe", "5432:5432"),
        ("hd-space-after", "5432:5432"),
        ("hd-space-quoted", "5432:5432"),
        ("if-guard", "5432:5432"),
        ("equals", "8080:80"),
        ("ip", "127.0.0.1:5432:5432"),
        ("ipv6", "[::1]:5432:5432"),
        ("known-flags", "127.0.0.1:5432:5432"),
        ("literal", "5432:5432"),
        ("long", "8080:80"),
        ("path", "5432:5432"),
        ("quoted", "5432:5432"),
        ("range", "8000-8002:8000-8002"),
        ("reservation", "5432:5432"),
        ("subshell", "5432:5432"),
        ("subst", "5432:5432"),
        ("subst-value", "5432:5432"),
        ("sudo", "5432:5432"),
        ("env-wrapper", "5432:5432"),
        ("timeout-wrapper", "5432:5432"),
        ("sudo-preserve", "5432:5432"),
        ("xargs-wrapper", "5432:5432"),
        ("xargs-equals", "5432:5432"),
        ("quiet-short", "5432:5432"),
        ("docker-create", "5432:5432"),
        ("docker-context", "5432:5432"),
        ("let-shift", "5432:5432"),
        ("quoted-arith", "5432:5432"),
        ("bash-c", "5434:5432"),
        ("sh-c", "5435:5432"),
        ("bash-lc", "5436:5432"),
        ("sudo-sh-c", "5469:5432"),
        ("eval-wrapper", "5449:5432"),
        ("sudo-end", "5440:5432"),
        ("sudo-user", "5474:5432"),
        ("env-path", "5473:5432"),
        ("env-chdir", "5445:5432"),
        ("env-split", "5444:5432"),
        ("timeout-signal", "5446:5432"),
        ("docker-tls", "5441:5432"),
        ("docker-tlsverify", "5443:5432"),
        ("docker-short-context", "5442:5432"),
        ("two", "5432:5432"),
        ("two", "6379:6379"),
        ("udp", "5432:5432/udp"),
        ("podman", "5432:5432"),
        ("nerdctl", "5432:5432"),
        ("arith-shift", "5432:5432"),
        ("dyn-after-publish", "5432:5432"),
    ] {
        expected.push(format!("ci.yml:{job}:1:{spec}"));
    }
    expected.push("ci.yml:same-indent-ports:0:6001:5432".to_string());
    expected.push("ci.yml:flow-services:0:6002:5432".to_string());
    expected.push("ci.yml:flow-nested:0:6003:5432".to_string());
    expected.push("ci.yml:flow-container:0:6007:80".to_string());
    expected.push("ci.yml:service-options:0:6010:5432".to_string());
    expected.push("ci.yml:service-ports:0:5432:5432".to_string());
    expected.push("ci.yml:service-ports:0:9:9".to_string());
    expected.push("ci.yml:service-ports:0:6:6".to_string());
    expected.push("ci.yml:service-ports:0:8080:80".to_string());
    expected.push("ci.yml:second-step:2:5432:5432".to_string());
    expected.push("compact.yml:a:1:5432:5432".to_string());
    expected.push("compact.yml:b:2:127.0.0.1:5432:5432".to_string());
    expected.push("action.yml:composite:1:5432:5432".to_string());
    expected.sort();
    assert_eq!(got, expected, "{findings:?}");
    let service_ports: Vec<_> = findings
        .iter()
        .filter(|finding| finding.target.as_deref() == Some("service-ports"))
        .map(|finding| finding.message.as_str())
        .collect();
    assert!(service_ports
        .iter()
        .any(|message| { message.contains("service `scalar`") && message.contains("in `9:9`") }));
    assert!(service_ports
        .iter()
        .any(|message| { message.contains("service `bare`") && message.contains("in `6:6`") }));
    assert!(service_ports
        .iter()
        .all(|message| !message.contains("in `8`")));

    let text = |file: &str| std::fs::read_to_string(root.join(file)).unwrap();
    let located = |file: &str, target: &str, step: usize| {
        findings
            .iter()
            .find(|finding| {
                finding.file.ends_with(file)
                    && finding.target.as_deref() == Some(target)
                    && super::step_of(&finding.message) == step
            })
            .unwrap()
            .line
    };
    let compact = text(".github/workflows/compact.yml");
    assert!(super::nth_line(&compact, located("compact.yml", "a", 1)).starts_with("    - run:"));
    assert!(super::nth_line(&compact, located("compact.yml", "b", 2)).contains("--publish"));
    let ci = text(".github/workflows/ci.yml");
    assert!(super::nth_line(&ci, located("ci.yml", "block", 1)).contains("docker run -p 5432:5432"));
    assert!(super::nth_line(&ci, located("ci.yml", "continued", 1))
        .trim()
        .starts_with("5432:5432"));
    assert!(super::nth_line(&ci, located("ci.yml", "literal", 1)).contains("- run:"));
    for job in [
        "hd-and",
        "hd-pipe",
        "hd-space-after",
        "hd-space-quoted",
        "arith-shift",
    ] {
        let line = super::nth_line(&ci, located("ci.yml", job, 1));
        assert!(line.contains("docker run"), "{job} {line}");
    }
    for finding in &findings {
        assert!(finding.message.contains("Publish a Docker-allocated port"));
    }

    let (root, config, files) = super::load_fixture("flow");
    let findings = super::super::check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 1);
    assert_eq!(findings[0].target.as_deref(), Some("build"));
    assert!(super::spec_of(&findings[0].message) == "5432:5432");
}
