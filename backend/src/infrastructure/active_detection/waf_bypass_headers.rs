use reqwest::RequestBuilder;
use rand::seq::IndexedRandom;

const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/114.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/114.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.5 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36 Edg/114.0.1823.51",
];

const BYPASS_IPS: &[&str] = &[
    "127.0.0.1",
    "192.168.0.1",
    "10.0.0.1",
    "8.8.8.8",
    "1.1.1.1",
];

/// Applies WAF bypass headers to a RequestBuilder to evade basic detection mechanisms.
pub fn apply_waf_bypass(mut builder: RequestBuilder) -> RequestBuilder {
    let mut rng = rand::rng();

    let user_agent = USER_AGENTS.choose(&mut rng).unwrap_or(&USER_AGENTS[0]);
    let bypass_ip = BYPASS_IPS.choose(&mut rng).unwrap_or(&BYPASS_IPS[0]);

    builder = builder
        .header("User-Agent", *user_agent)
        .header("X-Forwarded-For", *bypass_ip)
        .header("X-Originating-IP", *bypass_ip)
        .header("X-Remote-IP", *bypass_ip)
        .header("X-Remote-Addr", *bypass_ip)
        .header("X-Client-IP", *bypass_ip)
        .header("X-Host", "127.0.0.1")
        .header("X-Forwarded-Host", "127.0.0.1");

    builder
}
