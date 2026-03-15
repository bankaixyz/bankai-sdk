pub mod blocks;
pub mod chains;
pub mod ethereum;
pub mod health;
pub mod op_stack;

use crate::compat::case::CompatCaseDef;

pub fn cases() -> Vec<CompatCaseDef> {
    let mut cases = Vec::new();
    cases.extend(health::cases());
    cases.extend(chains::cases());
    cases.extend(blocks::cases());
    cases.extend(ethereum::cases());
    cases.extend(op_stack::cases());
    cases
}
