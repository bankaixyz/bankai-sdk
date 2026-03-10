use std::sync::OnceLock;
use std::time::Instant;

use crate::errors::SdkResult;

static SDK_DEBUG_ENABLED: OnceLock<bool> = OnceLock::new();

pub(crate) fn enabled() -> bool {
    *SDK_DEBUG_ENABLED.get_or_init(|| {
        std::env::var("BANKAI_SDK_DEBUG")
            .map(|value| {
                !matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "" | "0" | "false" | "no" | "off"
                )
            })
            .unwrap_or(false)
    })
}

pub(crate) fn log(message: impl AsRef<str>) {
    if enabled() {
        eprintln!("[bankai-sdk] {}", message.as_ref());
    }
}

pub(crate) fn elapsed_ms(start: Instant) -> u128 {
    start.elapsed().as_millis()
}

pub(crate) fn log_result<T>(label: impl AsRef<str>, start: Instant, result: &SdkResult<T>) {
    if !enabled() {
        return;
    }

    let label = label.as_ref();
    match result {
        Ok(_) => log(format!("{label} ok in {} ms", elapsed_ms(start))),
        Err(error) => log(format!("{label} failed in {} ms: {error}", elapsed_ms(start))),
    }
}

pub(crate) fn endpoint_label(url: &str) -> String {
    match url::Url::parse(url) {
        Ok(parsed) => {
            let host = parsed.host_str().unwrap_or("unknown-host");
            match parsed.port() {
                Some(port) => format!("{}://{}:{}", parsed.scheme(), host, port),
                None => format!("{}://{}", parsed.scheme(), host),
            }
        }
        Err(_) => "<invalid-url>".to_string(),
    }
}
