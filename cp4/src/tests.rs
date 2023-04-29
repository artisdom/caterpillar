use std::collections::BTreeMap;

use crate::{cp, test_report::TestReport};

pub fn run() -> Vec<TestReport> {
    let mut tests = BTreeMap::new();
    let mut test_reports = Vec::new();

    tests.insert(("bool", "true"), "true");
    tests.insert(("bool", "false not"), "false not");

    for ((module, name), code) in tests {
        let module = module.into();
        let name = name.into();

        let mut data_stack = cp::DataStack::new();
        let result = cp::execute(code, &mut data_stack);

        test_reports.push(TestReport::new(module, name, result, data_stack));
    }

    test_reports
}
