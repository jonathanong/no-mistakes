use super::super::mapping::{classify, Verdict};

#[test]
fn parser_edges_cover_wrappers_hosts_and_yaml_shapes() {
    for spec in [
        "",
        "1 2",
        "80/http",
        "80/tcp/udp",
        "[]:1:2",
        "a:b:c:d",
        "[::1]:a:b:c",
        "${unclosed:80",
        "$:80",
        "${{PORT}}x:80",
        "${{a${{b}}}}:80",
        "${{ PORT:80",
        "0-0-0:80",
        "1-2-3:80",
        "${a{b}}:80",
        "${}:80",
    ] {
        assert!(matches!(classify(spec), Verdict::Silent), "{spec}");
    }
    for spec in ["[::1]::80", "80", "0-0:80", "00:80", "${{PORT}}:80"] {
        assert!(matches!(classify(spec), Verdict::Allowed), "{spec}");
    }
    assert!(matches!(classify("9:8/sctp"), Verdict::Literal(_)));
    assert!(matches!(classify("[::1]:9:80"), Verdict::Literal(_)));

    assert_eq!(super::super::flags::short_kind('z'), None);
    assert_eq!(super::super::flags::short_kind('q'), Some(false));
    assert_eq!(super::super::flags::long_arg("nope"), None);
    assert!(!super::super::flags::is_assign("1=x"));
    assert!(!super::super::flags::is_assign("echo"));
    assert_eq!(super::super::flags::basename(r"C:\docker"), "docker");

    let hit = |script: &str| super::super::publish::publish_hits(script);
    assert!(hit("docker run -p 1:2 \"").is_empty());
    assert!(hit("docker build -p 1:2 img").is_empty());
    assert!(hit("docker container build -p 1:2 img").is_empty());
    assert!(hit("docker run -z -p 1:2 img").is_empty());
    assert!(hit("docker run --publish").is_empty());
    assert_eq!(hit("docker run --mystery=1 -p 1:2 img").len(), 1);
    assert_eq!(hit("docker run --rm=true -p 3:4 img").len(), 1);
    assert_eq!(hit("docker run --memory $(echo 1) -p 5:6 img").len(), 1);
    assert_eq!(hit("docker run -p=7:8 img").len(), 1);
    assert_eq!(hit("sudo -E -n -H -u root docker run -p 1:2 img").len(), 1);
    assert!(hit("sudo -u").is_empty());
    assert_eq!(hit("sudo --preserve-env docker run -p 1:2 img").len(), 1);
    assert_eq!(
        hit("sudo --preserve-env=PATH docker run -p 1:2 img").len(),
        1
    );
    assert_eq!(hit("env FOO=1 docker run -p 1:2 img").len(), 1);
    assert_eq!(hit("env -i -u NAME docker run -p 1:2 img").len(), 1);
    assert_eq!(
        hit("env --ignore-environment --unset=FOO docker run -p 1:2 img").len(),
        1
    );
    assert_eq!(hit("timeout 60 docker run -p 1:2 img").len(), 1);
    assert_eq!(
        hit("timeout --foreground 60s docker run -p 1:2 img").len(),
        1
    );
    assert_eq!(hit("nice -n 5 docker run -p 1:2 img").len(), 1);
    assert_eq!(hit("stdbuf -oL docker run -p 1:2 img").len(), 1);
    assert_eq!(hit("echo pg | xargs docker run -p 1:2 img").len(), 1);
    assert_eq!(hit("echo pg | xargs -n 1 docker run -p 1:2 img").len(), 1);
    assert_eq!(hit("echo pg | xargs -t docker run -p 1:2 img").len(), 1);
    assert_eq!(hit("docker --debug run -p 1:2 img").len(), 1);
    assert_eq!(hit("docker run -q -p 1:2 img").len(), 1);
    assert_eq!(hit("docker create -p 1:2 img").len(), 1);
    assert_eq!(hit("docker container create -p 1:2 img").len(), 1);
    assert_eq!(hit("docker --context ci run -p 1:2 img").len(), 1);
    assert_eq!(hit("docker --context=ci create -p 1:2 img").len(), 1);
    assert_eq!(
        hit("docker -D --log-level info --config /tmp -H tcp://127.0.0.1:2375 run -p 1:2 img")
            .len(),
        1
    );
    assert_eq!(hit("() docker run -p 1:2 img").len(), 1);
    assert_eq!(hit("(FOO=1) docker run -p 2:3 img").len(), 1);
    assert_eq!(hit("time ! { docker run -p 4:5 img; }").len(), 1);
    assert_eq!(
        hit("cat <<\"EOF\"\ninner\nEOF\ndocker run -p 6:7 img").len(),
        1
    );
    assert_eq!(hit("cat << EOF\nx\nEOF\ndocker run -p 1:2 img").len(), 1);
    assert!(hit("cat<<EOF\ndocker run -p 1:2 img\nEOF").is_empty());
    assert_eq!(
        hit("cat <<-EOF\n\tinner\nEOF\ndocker run -p 8:9 img").len(),
        1
    );
    assert!(hit("cat <<EOF\ninner").is_empty());
    assert_eq!(hit("echo <<<text\ndocker run -p 1:2 img").len(), 1);
    assert_eq!(hit("echo <(cat) >(tee)\ndocker run -p 1:2 img").len(), 1);
    assert!(hit("echo `unterminated").is_empty());
    assert!(hit("echo $(echo hi").is_empty());
    assert_eq!(hit("docker run -p 1:2 $(echo").len(), 1);
    assert_eq!(hit("docker run -p 1:2 `echo").len(), 1);
    assert!(hit("docker run -p $(echo").is_empty());
    assert_eq!(hit("(( n = 1 << 3 ))\ndocker run -p 1:2 img").len(), 1);
    assert_eq!(hit("((n=1<<3))\ndocker run -p 1:2 img").len(), 1);
    assert_eq!(hit("let n=1<<3\ndocker run -p 1:2 img").len(), 1);
    let quoted_arith = hit("cat \"((\" <<EOF\ndocker run -p 1:2 a\nEOF\ndocker run -p 3:4 b");
    assert_eq!(quoted_arith.len(), 1);
    assert_eq!(quoted_arith[0].spec, "3:4");
    let quoted = super::super::shell::commands("echo $(echo 'a)' \"b)\")\necho \"a\nb\" \"trail\\");
    assert!(quoted.len() >= 2);
    assert!(quoted.iter().any(|command| command.dynamic));

    let numbered = "\
on: push
jobs:
  1:
    steps:
      - run: docker run -p 1:2 img
";
    let findings = super::check(super::WORKFLOW, numbered);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("1"));

    let boolean = "\
on: push
jobs:
  true:
    steps:
      - run: docker run -p 1:2 img
";
    assert_eq!(
        super::check(super::WORKFLOW, boolean)[0].target.as_deref(),
        Some("true")
    );

    let empty_name = "\
on: push
jobs:
  \"\":
    steps:
      - run: docker run -p 1:2 img
";
    assert_eq!(
        super::check(super::WORKFLOW, empty_name)[0]
            .target
            .as_deref(),
        Some("job")
    );

    for source in [
        "on: push\njobs:\n  build: null\n",
        "on: push\njobs:\n  build:\n    runs-on: x\n",
        "on: push\njobs:\n  build:\n    steps:\n      - 1\n",
        "name: a\nruns:\n  using: composite\n",
        "name: a\nruns:\n  using: composite\n  steps:\n    - uses: actions/checkout@v4\n",
    ] {
        let rel = if source.contains("jobs:") {
            super::WORKFLOW
        } else {
            super::ACTION
        };
        assert!(super::check(rel, source).is_empty(), "{source}");
    }

    let folded = "\
on: push
jobs:
  build:
    steps:
      - run: >
          docker run -p
          5432:5432 postgres
";
    let findings = super::check(super::WORKFLOW, folded);
    assert_eq!(findings.len(), 1, "{findings:?}");
    let line = folded.lines().nth(findings[0].line - 1).unwrap();
    assert!(line.contains("5432:5432"), "{line}");

    let services = "\
on: push
jobs:
  svc:
    services:
      pg:
        ports: [\"9:9\", \"$PORT:1\"]

      extra:
        ports:
          - \"8:8\" # pinned
    container: ubuntu
    steps:
      - run: echo ok
";
    let findings = super::check(super::WORKFLOW, services);
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings.iter().any(|finding| {
        finding.line == 6
            && finding.message.contains("service `pg`")
            && finding.message.contains("in `9:9`")
    }));
    assert!(findings.iter().any(|finding| {
        finding.message.contains("service `extra`") && finding.message.contains("in `8:8`")
    }));
}
