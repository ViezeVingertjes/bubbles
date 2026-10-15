use super::DialogueRng;

#[test]
fn same_seed_replays_the_same_stream() {
    let a = DialogueRng::from_seed(1234);
    let b = DialogueRng::from_seed(1234);
    for _ in 0..64 {
        assert_eq!(a.next_u64(), b.next_u64());
    }
}

#[test]
fn different_seeds_diverge() {
    let a = DialogueRng::from_seed(1);
    let b = DialogueRng::from_seed(2);
    assert_ne!(a.next_u64(), b.next_u64());
}

/// Guards the stream against accidental reimplementation: these are the first
/// four `SplitMix64` outputs for seed 0, and they are part of the save format.
#[test]
fn stream_is_pinned_to_known_values() {
    let rng = DialogueRng::from_seed(0);
    let got: Vec<u64> = (0..4).map(|_| rng.next_u64()).collect();
    assert_eq!(
        got,
        vec![
            0xE220_A839_7B1D_CDAF,
            0x6E78_9E6A_A1B9_65F4,
            0x06C4_5D18_8009_454F,
            0xF88B_B8A8_724C_81EC,
        ]
    );
}

#[test]
fn state_round_trips_and_resumes_the_stream() {
    let rng = DialogueRng::from_seed(7);
    let _ = rng.next_u64();
    let saved = rng.state();
    let expected: Vec<u64> = (0..4).map(|_| rng.next_u64()).collect();

    let resumed = DialogueRng::from_seed(0);
    resumed.set_state(saved);
    assert_eq!(
        expected,
        (0..4).map(|_| resumed.next_u64()).collect::<Vec<_>>()
    );
}

#[test]
fn clones_share_one_stream() {
    let rng = DialogueRng::from_seed(99);
    let handle = rng.clone();
    let first = rng.next_u64();
    assert_ne!(first, handle.next_u64());
    assert_eq!(rng.state(), handle.state());
}

#[test]
fn reseed_restarts_the_stream() {
    let rng = DialogueRng::from_seed(5);
    let first = rng.next_u64();
    rng.reseed(5);
    assert_eq!(first, rng.next_u64());
}

#[test]
fn next_f64_stays_in_unit_interval() {
    let rng = DialogueRng::from_seed(3);
    for _ in 0..10_000 {
        let v = rng.next_f64();
        assert!((0.0..1.0).contains(&v), "{v} out of range");
    }
}

#[test]
fn choose_index_is_none_when_empty_and_in_range_otherwise() {
    let rng = DialogueRng::from_seed(11);
    assert_eq!(rng.choose_index(0), None);
    for _ in 0..1_000 {
        assert!(rng.choose_index(3).is_some_and(|i| i < 3));
    }
}

#[test]
fn choose_index_covers_every_slot() {
    let rng = DialogueRng::from_seed(12);
    let mut seen = [false; 4];
    for _ in 0..1_000 {
        seen[rng.choose_index(4).unwrap()] = true;
    }
    assert!(seen.iter().all(|s| *s));
}

#[test]
fn range_inclusive_covers_both_ends_and_nothing_else() {
    let rng = DialogueRng::from_seed(13);
    let mut low = false;
    let mut high = false;
    for _ in 0..1_000 {
        let v = rng.range_inclusive(-2, 2);
        assert!((-2..=2).contains(&v), "{v} out of range");
        low |= v == -2;
        high |= v == 2;
    }
    assert!(low && high);
}

#[test]
fn range_inclusive_handles_degenerate_and_extreme_bounds() {
    let rng = DialogueRng::from_seed(14);
    assert_eq!(rng.range_inclusive(4, 4), 4);
    assert_eq!(rng.range_inclusive(9, 1), 9);
    let v = rng.range_inclusive(i64::MIN, i64::MAX);
    assert!((i64::MIN..=i64::MAX).contains(&v));
}

#[test]
fn unseeded_generators_differ() {
    let a = DialogueRng::from_entropy();
    let b = DialogueRng::from_entropy();
    assert_ne!(a.next_u64(), b.next_u64());
}
