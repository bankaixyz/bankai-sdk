pub mod api;
pub mod assertions;
pub mod case;
pub mod context;
pub mod openapi_minimal;
pub mod runner;

use case::CompatCaseDef;

pub fn all_cases() -> Vec<CompatCaseDef> {
    api::cases()
}
