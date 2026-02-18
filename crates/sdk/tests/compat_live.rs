mod compat;

use compat::case::CompatArea;
use compat::context::CompatContext;
use compat::runner::{assert_reports, case_in_phase, run_case, SuitePhase};

#[tokio::test]
#[ignore = "live compatibility suite; requires running API"]
async fn compat_live_decode_suite() {
    run_suite(SuitePhase::Decode, "decode").await;
}

#[tokio::test]
#[ignore = "live compatibility suite; requires running API"]
async fn compat_live_verify_suite() {
    run_suite(SuitePhase::Verify, "verify").await;
}

async fn run_suite(phase: SuitePhase, suite_name: &str) {
    let ctx = CompatContext::from_env();
    let cases: Vec<_> = compat::all_cases()
        .into_iter()
        .filter(|case| case_in_phase(case, phase))
        .collect();
    let total = cases.len();
    let verbose = std::env::var("COMPAT_VERBOSE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let mut reports = Vec::with_capacity(total);
    for (index, case) in cases.into_iter().enumerate() {
        if verbose {
            eprintln!(
                "[{}/{}] running [{}] {}",
                index + 1,
                total,
                area_label(case.area),
                case.id.0
            );
        }
        let report = run_case(&ctx, case).await;
        if verbose {
            eprintln!(
                "[{}/{}] result [{}] {} => {:?}",
                index + 1,
                total,
                area_label(report.area),
                report.id.0,
                report.status
            );
        }
        reports.push(report);
    }

    assert_reports(suite_name, &reports);
}

#[tokio::test]
#[ignore = "live compatibility suite; requires running API"]
async fn compat_live_openapi_coverage() {
    let ctx = CompatContext::from_env();
    compat::openapi_minimal::run(&ctx)
        .await
        .expect("openapi endpoint coverage check failed");
}

fn area_label(area: CompatArea) -> &'static str {
    match area {
        CompatArea::Health => "health",
        CompatArea::Chains => "chains",
        CompatArea::Blocks => "blocks",
        CompatArea::Stats => "stats",
        CompatArea::EthereumBeacon => "ethereum_beacon",
        CompatArea::EthereumExecution => "ethereum_execution",
        CompatArea::EthereumRoot => "ethereum_root",
    }
}
