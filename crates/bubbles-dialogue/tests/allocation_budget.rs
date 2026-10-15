//! Allocation accounting for the runner's hot paths.
//!
//! The gap: the guide stated "the runner never allocates on its own" and
//! promised "allocation-free `{$var}` interpolation" to any storage that
//! overrides `get_ref`. Both were unmeasured, and both were wrong - a line event
//! owns its text, so emitting one always allocates. Nothing in a suite of 400+
//! tests could tell, because none of them could see an allocation.
//!
//! What this file does instead of restating a slogan: it puts a **budget** on
//! each hot path. Budgets are upper bounds, so a genuine improvement does not
//! fail the suite, but adding a clone to the per-event path does. The numbers
//! are allocation *counts*, not timings, so they are machine-independent and
//! safe to assert on in CI - unlike the figures in `benches/`.
//!
//! All measurement happens in a single `#[test]` because the counter is global.
//! The test harness runs tests in one binary concurrently, so a second test in
//! this file would have its allocations folded into the first one's count.

// A counting global allocator cannot be written in safe Rust, and the crate
// denies unsafe everywhere else. This is a dev-only test binary and the unsafe
// is confined to forwarding three calls to `System`.
#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use bubbles::{HashMapStorage, Runner, Value, VariableStorage, compile, compile_many};

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
static COUNTING: AtomicBool = AtomicBool::new(false);

/// Forwards everything to the system allocator, counting allocations while
/// [`COUNTING`] is set.
struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // A realloc that grows a String is a real allocation for budgeting
        // purposes: it is the cost the caller pays for an under-sized buffer.
        if COUNTING.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            BYTES.fetch_add(new_size.saturating_sub(layout.size()), Ordering::Relaxed);
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// Runs `f` with allocation counting on, returning `(allocations, bytes, value)`.
fn measure<T>(f: impl FnOnce() -> T) -> (usize, usize, T) {
    ALLOCATIONS.store(0, Ordering::Relaxed);
    BYTES.store(0, Ordering::Relaxed);
    COUNTING.store(true, Ordering::Relaxed);
    let value = f();
    COUNTING.store(false, Ordering::Relaxed);
    (
        ALLOCATIONS.load(Ordering::Relaxed),
        BYTES.load(Ordering::Relaxed),
        value,
    )
}

/// Storage that does not override `get_ref`, so reads go through the default
/// `Cow::Owned` path.
#[derive(Default)]
struct OwnedOnlyStorage {
    map: HashMap<String, Value>,
}

impl VariableStorage for OwnedOnlyStorage {
    fn get(&self, name: &str) -> Option<Value> {
        self.map.get(name).cloned()
    }
    fn set(&mut self, name: &str, value: Value) {
        self.map.insert(name.to_owned(), value);
    }
}

/// Same data, but `get_ref` overridden to borrow - the optimisation the guide
/// used to promise would make interpolation allocation-free.
#[derive(Default)]
struct BorrowingStorage {
    map: HashMap<String, Value>,
}

impl VariableStorage for BorrowingStorage {
    fn get(&self, name: &str) -> Option<Value> {
        self.map.get(name).cloned()
    }
    fn set(&mut self, name: &str, value: Value) {
        self.map.insert(name.to_owned(), value);
    }
    fn get_ref(&self, name: &str) -> Option<Cow<'_, Value>> {
        self.map.get(name).map(Cow::Borrowed)
    }
}

fn seed<S: VariableStorage>(storage: &mut S) {
    storage.set("$name", Value::Text("Aurelia the Navigator".into()));
    storage.set("$gold", Value::Number(42.0));
    storage.set("$port", Value::Text("Tide's End".into()));
}

/// Budget: allocations permitted for one `next_event` returning a plain line.
///
/// A line event owns its `text: String`, so one allocation is structural. The
/// budget leaves no headroom above that deliberately - this is the single most
/// frequently executed path in the library.
const PLAIN_LINE_BUDGET: usize = 1;

/// Budget for one `next_event` on a line with three interpolations and one
/// arithmetic expression. Measured at 6; the slack absorbs a `String` growth
/// step without hiding a new clone per segment.
const INTERPOLATED_LINE_BUDGET: usize = 8;

/// Budget for `snapshot()` on a session with a handful of visits. Measured at 2
/// (the visits map and the once-seen set).
const SNAPSHOT_BUDGET: usize = 4;

/// The `examples/harbour/` sources, embedded so no measured region does file IO.
const HARBOUR: [(&str, &str); 2] = [
    (
        "harbour.bub",
        include_str!("../../../examples/harbour/harbour.bub"),
    ),
    (
        "services.bub",
        include_str!("../../../examples/harbour/services.bub"),
    ),
];

#[test]
fn hot_paths_stay_within_allocation_budget() {
    // ── a plain line event ────────────────────────────────────────────────────
    let mut src = String::from("title: P\n---\n");
    for i in 0..64 {
        let _ = writeln!(src, "Line number {i} of plain text with no interpolation.");
    }
    src.push_str("===\n");
    let program = compile(&src).unwrap();
    let mut runner = Runner::new(program, HashMapStorage::new());
    runner.start("P").unwrap();
    let _ = runner.next_event().unwrap(); // NodeStarted
    let _ = runner.next_event().unwrap(); // first Line, warms any lazy growth

    let (allocations, _, event) = measure(|| runner.next_event().unwrap());
    assert!(event.is_some(), "expected a line event");
    assert!(
        allocations <= PLAIN_LINE_BUDGET,
        "a plain line event took {allocations} allocations, budget is {PLAIN_LINE_BUDGET}"
    );
    // The counterpart assertion: it is not zero, and cannot be while the event
    // owns its text. This is what falsifies "the runner never allocates on its
    // own", and it fails if anyone restores that claim without changing the API.
    assert!(
        allocations > 0,
        "a line event allocates its text; if this is now zero the event type changed \
         and the allocation claims in book/src/getting-started/event-loop.md can be revisited"
    );

    // ── interpolation, and whether get_ref actually helps ─────────────────────
    let interp = compile(
        "title: I\n---\nClerk: {$name}, {$gold} gold at {$port}, {$gold * 2} by spring.\n===\n",
    )
    .unwrap();

    let mut owned = OwnedOnlyStorage::default();
    seed(&mut owned);
    let mut default_path = Runner::new(interp.clone(), owned);
    default_path.start("I").unwrap();
    let _ = default_path.next_event().unwrap();
    let (default_allocations, _, default_event) = measure(|| default_path.next_event().unwrap());

    let mut borrowing = BorrowingStorage::default();
    seed(&mut borrowing);
    let mut borrowed_path = Runner::new(interp, borrowing);
    borrowed_path.start("I").unwrap();
    let _ = borrowed_path.next_event().unwrap();
    let (borrowed_allocations, _, borrowed_event) = measure(|| borrowed_path.next_event().unwrap());

    assert_eq!(
        default_event, borrowed_event,
        "the two storages must render the same line"
    );
    assert!(
        default_allocations <= INTERPOLATED_LINE_BUDGET,
        "interpolated line took {default_allocations} allocations, \
         budget is {INTERPOLATED_LINE_BUDGET}"
    );

    // Overriding `get_ref` currently changes nothing, because the evaluator calls
    // `Cow::into_owned` on the result (runtime/eval/mod.rs) and clones the value
    // one line after borrowing it. If this assertion ever fails because the
    // borrowed path got cheaper, the optimisation has become real and the
    // `get_ref` guidance in book/src/integration/storage.md can promise it again.
    assert_eq!(
        borrowed_allocations, default_allocations,
        "borrowing get_ref changed the allocation count; if it is now lower, the \
         evaluator stopped cloning and the storage guide should be updated to match"
    );

    // ── snapshot and restore ─────────────────────────────────────────────────
    let chained = compile(
        "title: A\n---\nOne.\n<<jump B>>\n===\ntitle: B\n---\nTwo.\n<<jump C>>\n===\ntitle: C\n---\nThree.\n===\n",
    )
    .unwrap();
    let mut session = Runner::new(chained.clone(), HashMapStorage::new());
    session.start("A").unwrap();
    while session.next_event().unwrap().is_some() {}

    let (snapshot_allocations, _, snapshot) = measure(|| session.snapshot());
    assert!(
        snapshot_allocations <= SNAPSHOT_BUDGET,
        "snapshot took {snapshot_allocations} allocations, budget is {SNAPSHOT_BUDGET}"
    );

    let mut target = Runner::new(chained, HashMapStorage::new());
    let (restore_allocations, _, result) = measure(|| target.restore(snapshot));
    result.unwrap();
    assert_eq!(
        restore_allocations, 0,
        "restore moves the snapshot's maps into the runner and should not allocate"
    );

    // ── program clone against compile ────────────────────────────────────────
    // The guide calls a Program "cheap to clone" and compiling "the expensive
    // thing". In allocation terms that ratio is stable on any machine, unlike
    // the wall clock in benches/, so it is safe to assert here.
    let (compile_allocations, _, harbour) = measure(|| compile_many(&HARBOUR).unwrap());
    let (clone_allocations, _, cloned) = measure(|| harbour.clone());
    assert_eq!(
        harbour.node_titles().count(),
        cloned.node_titles().count(),
        "clone must preserve the node table"
    );
    assert!(
        clone_allocations * 4 <= compile_allocations,
        "cloning a Program took {clone_allocations} allocations against {compile_allocations} \
         to compile it; the guide's claim that programs are the expensive thing and clones are \
         cheap needs at least a 4x gap to hold"
    );
}
