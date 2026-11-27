use regex::Regex;
use std::sync::OnceLock;

pub(super) fn python() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?x)\b(?:requests|httpx)\.(?:get|post|put|patch|delete|head|options)\(\s*['"]([^'"]+)['"]\s*[,)]"#,
        )
        .expect("python client")
    })
}

pub(super) fn go_get() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"\bhttp\.(?:Get|Post|Head|PostForm)\(\s*(?:"([^"]+)"|`([^`]+)`)[ \t]*[,)]"#)
            .expect("go client")
    })
}

pub(super) fn go_request() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"\bhttp\.NewRequest\(\s*(?:"[A-Z]+"|http\.Method[A-Za-z]+)\s*,\s*(?:"([^"]+)"|`([^`]+)`)[ \t]*[,)]"#,
        )
        .expect("go request")
    })
}

pub(super) fn go_request_ctx() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"\bhttp\.NewRequestWithContext\(\s*[^,]+,\s*(?:"[A-Z]+"|http\.Method[A-Za-z]+)\s*,\s*(?:"([^"]+)"|`([^`]+)`)[ \t]*[,)]"#,
        )
        .expect("go request context")
    })
}

pub(super) fn rust_get() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"\breqwest::get\(\s*"([^"]+)"\s*[,)]"#).expect("reqwest"))
}

pub(super) fn rust_client() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"\breqwest::Client::new\(\)\.(?:get|post|put|patch|delete)\(\s*"([^"]+)"\s*[,)]"#,
        )
        .expect("reqwest client")
    })
}

pub(super) fn ruby() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"\b(?:Faraday|HTTParty|RestClient)\.(?:get|post|put|patch|delete)\(\s*["']([^"']+)["']\s*[,)]"#,
        )
        .expect("ruby client")
    })
}

pub(super) fn ruby_net() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"\bNet::HTTP\.get\(\s*URI\(\s*["']([^"']+)["']\s*\)"#).expect("net http")
    })
}

pub(super) fn php() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"\bHttp::(?:get|post|put|patch|delete)\(\s*['"]([^'"]+)['"]\s*[,)]"#)
            .expect("laravel http")
    })
}

pub(super) fn php_request() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"->request\(\s*['"](?:GET|POST|PUT|PATCH|DELETE)['"]\s*,\s*['"]([^'"]+)['"]\s*[,)]"#,
        )
        .expect("php request")
    })
}

pub(super) fn jvm() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?x)
            \.(?:getForObject|getForEntity|postForObject|postForEntity|exchange)\(\s*"([^"]+)"\s*[,)]
            |\bURI\.create\(\s*"([^"]+)"\s*\)
            |\.uri\(\s*"([^"]+)"\s*[,)]
            "#,
        )
        .expect("jvm client")
    })
}

pub(super) fn elixir() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"\b(?:HTTPoison|Req|Tesla)\.(?:get|post|put|patch|delete)!?\(\s*"([^"]+)"\s*[,)]"#,
        )
        .expect("elixir client")
    })
}

pub(super) fn dotnet() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"\.(?:GetAsync|PostAsync|PutAsync|PatchAsync|DeleteAsync|GetFromJsonAsync|PostAsJsonAsync|PutAsJsonAsync|PatchAsJsonAsync)\(\s*"([^"]+)"\s*[,)]"#,
        )
        .expect("dotnet client")
    })
}

pub(super) fn dotnet_message() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"\bnew\s+HttpRequestMessage\(\s*HttpMethod\.\w+\s*,\s*"([^"]+)"\s*[,)]"#)
            .expect("dotnet message")
    })
}
