pub mod beacon;
pub mod execution;
pub mod root;

use crate::compat::case::CompatCaseDef;

pub fn cases() -> Vec<CompatCaseDef> {
    let mut cases = Vec::new();
    cases.extend(beacon::cases());
    cases.extend(execution::cases());
    cases.extend(root::cases());
    cases
}
