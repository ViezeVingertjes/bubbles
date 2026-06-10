//! End-to-end test for the harbour example scripts.

use bubbles::{DialogueEvent, HashMapStorage, Runner, RunnerPhase, compile_many};

#[test]
fn harbour_example_drains_to_dialogue_complete() {
    let harbour = include_str!("../../../examples/harbour/harbour.bub");
    let services = include_str!("../../../examples/harbour/services.bub");
    let prog = compile_many(&[("harbour.bub", harbour), ("services.bub", services)]).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Start").unwrap();
    let mut last = None;
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Options(opts) = &ev {
            let idx = opts
                .iter()
                .position(|o| o.available)
                .expect("at least one available option");
            runner.select_option(idx).unwrap();
        }
        last = Some(ev);
    }
    assert_eq!(runner.phase(), RunnerPhase::Done);
    assert!(matches!(last, Some(DialogueEvent::DialogueComplete)));
}
