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
//! are allocation *counts*, not timings, so they are stable enough to assert on
//! in CI - unlike the figures in `benches/`.
//!
//! "Machine-independent" needs one qualification, found by running these on
//! three platforms rather than assumed: a *steady-state* count is the same
//! everywhere, but a first call is not. The first time a session reads its
//! random stream, macOS charges one allocation for initialising std's mutex
//! that Linux and Windows do not. So every budget here is measured after the
//! path has been exercised once, and the warm-up is not incidental tidiness.
//!
//! The counters are also **per thread**, not global. A global counter attributes
//! every allocation made anywhere in the process during the measured window to
//! the closure being measured, including the test harness's own bookkeeping on
//! its main thread, which would make a zero-headroom budget a measurement of
//! how busy the machine was. That is latent rather than observed - the macOS
//! figure above was the cold lock, not a race - so the last test in this file
//! asserts the property instead of leaving it to be assumed.

// A counting global allocator cannot be written in safe Rust, and the crate
// denies unsafe everywhere else. This is a dev-only test binary and the unsafe
// is confined to forwarding three calls to `System`.
#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use bubbles::{HashMapStorage, Runner, Value, VariableStorage, compile, compile_many};

thread_local! {
    /// Allocations this thread has made since `measure` reset the counter.
    ///
    /// `const` initialisation matters: a lazily initialised thread-local would
    /// allocate on first access, from inside the allocator.
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    /// Bytes this thread has requested since `measure` reset the counter.
    static BYTES: Cell<usize> = const { Cell::new(0) };
    /// Whether this thread is inside a `measure` call.
    static COUNTING: Cell<bool> = const { Cell::new(false) };
}

/// Adds one allocation of `size` bytes to this thread's counters, if it is
/// counting. `try_with` rather than `with` so that an allocation during
/// thread-local teardown cannot panic inside the allocator.
fn record(size: usize) {
    if COUNTING.try_with(Cell::get).unwrap_or(false) {
        let _ = ALLOCATIONS.try_with(|c| c.set(c.get() + 1));
        let _ = BYTES.try_with(|c| c.set(c.get() + size));
    }
}

/// Forwards everything to the system allocator, counting the calling thread's
/// allocations while [`COUNTING`] is set for that thread.
struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record(layout.size());
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // A realloc that grows a String is a real allocation for budgeting
        // purposes: it is the cost the caller pays for an under-sized buffer.
        record(new_size.saturating_sub(layout.size()));
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// Runs `f` with allocation counting on for this thread, returning
/// `(allocations, bytes, value)`.
fn measure<T>(f: impl FnOnce() -> T) -> (usize, usize, T) {
    ALLOCATIONS.set(0);
    BYTES.set(0);
    COUNTING.set(true);
    let value = f();
    COUNTING.set(false);
    (ALLOCATIONS.get(), BYTES.get(), value)
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

/// Budget for `snapshot()` on a session that has visited three nodes. Measured
/// at 4: one node for the cloned visits map and one `String` per key. The
/// once-seen set is empty and the random stream's position is a plain `u64`, so
/// neither costs anything. No headroom, which is why this one is measured on a
/// warmed path and with a per-thread counter.
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

    // Both of these are measured on the second call, and the first call is
    // reported rather than asserted on. `snapshot` reads the random stream's
    // position and `restore` writes it, so each takes its runner's lock for the
    // first time, and on macOS a first lock costs one allocation that Linux and
    // Windows do not charge - std initialises its mutex lazily there. That is a
    // one-off per runner, not a per-save cost, so budgeting the first call would
    // be budgeting the platform rather than the code.
    let (first_snapshot_allocations, _, first) = measure(|| session.snapshot());
    drop(first);

    let (snapshot_allocations, _, snapshot) = measure(|| session.snapshot());
    assert!(
        snapshot_allocations <= SNAPSHOT_BUDGET,
        "snapshot took {snapshot_allocations} allocations, budget is {SNAPSHOT_BUDGET} \
         (the first snapshot of this session took {first_snapshot_allocations}; if both are \
         over budget, snapshot started allocating rather than a lock initialising)"
    );

    let mut target = Runner::new(chained, HashMapStorage::new());
    let (first_restore_allocations, _, warm_up) = measure(|| target.restore(session.snapshot()));
    warm_up.unwrap();

    let (restore_allocations, _, result) = measure(|| target.restore(snapshot));
    result.unwrap();
    assert_eq!(
        restore_allocations, 0,
        "restore moves the snapshot's maps into the runner and should not allocate; the first \
         restore into this runner took {first_restore_allocations}"
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

/// The gap: the budgets above are only meaningful if a measured figure is a
/// property of the measured code. A process-global counter is not - it also
/// counts whatever else the process allocates in the same window, which is how
/// a zero-headroom budget came to fail on one platform of three while being
/// reproducibly exact on another. Nothing could observe that from inside a
/// single-threaded measurement, so this measures a known cost while a second
/// thread allocates as fast as it can.
#[test]
fn allocation_counting_sees_only_the_measuring_thread() {
    const EXPECTED: usize = 1_000;

    let stop = Arc::new(AtomicBool::new(false));
    let started = Arc::new(AtomicBool::new(false));
    let noise = std::thread::spawn({
        let (stop, started) = (Arc::clone(&stop), Arc::clone(&started));
        move || {
            while !stop.load(Ordering::Relaxed) {
                drop(std::hint::black_box(String::from("on another thread")));
                started.store(true, Ordering::Relaxed);
            }
        }
    });
    while !started.load(Ordering::Relaxed) {
        std::hint::spin_loop();
    }

    // One `String::from` of a non-empty literal is exactly one allocation: the
    // capacity is known up front, so there is no growth step to blur the count.
    let (allocations, _, strings) = measure(|| {
        let mut made = 0;
        for _ in 0..EXPECTED {
            drop(std::hint::black_box(String::from("one allocation")));
            made += 1;
        }
        made
    });

    stop.store(true, Ordering::Relaxed);
    noise.join().unwrap();

    assert_eq!(strings, EXPECTED, "the measured closure must have run");
    assert_eq!(
        allocations, EXPECTED,
        "measured {allocations} allocations for {EXPECTED} strings while another thread \
         allocated throughout; the counter is counting the whole process again, which makes \
         every budget in this file a measurement of how busy the machine was"
    );
}
