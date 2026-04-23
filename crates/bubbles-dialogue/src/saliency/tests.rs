use super::{BestLeastRecentlyViewed, Candidate, FirstAvailable, SaliencyStrategy};

#[cfg(feature = "rand")]
use super::RandomAvailable;

fn cand(ids: &[&'static str]) -> Vec<Candidate<'static>> {
    ids.iter()
        .map(|&id| Candidate {
            id,
            available: true,
        })
        .collect()
}

fn cand_mask(ids: &[&'static str], mask: &[bool]) -> Vec<Candidate<'static>> {
    ids.iter()
        .zip(mask.iter())
        .map(|(&id, &available)| Candidate { id, available })
        .collect()
}

// ── FirstAvailable ───────────────────────────────────────────────────────

#[test]
fn first_available_picks_first() {
    let mut s = FirstAvailable;
    let cs = cand_mask(&["a", "b", "c"], &[false, true, true]);
    assert_eq!(s.select(&cs), Some(1));
}

#[test]
fn first_available_none_when_all_unavailable() {
    let mut s = FirstAvailable;
    let cs = cand_mask(&["a", "b"], &[false, false]);
    assert_eq!(s.select(&cs), None);
}

#[test]
fn first_available_empty() {
    let mut s = FirstAvailable;
    assert_eq!(s.select(&[]), None);
}

// ── RandomAvailable ──────────────────────────────────────────────────────

#[cfg(feature = "rand")]
#[test]
fn random_available_returns_valid_index() {
    let mut s = RandomAvailable;
    let cs = cand_mask(&["a", "b", "c"], &[true, false, true]);
    let idx = s.select(&cs);
    assert!(idx == Some(0) || idx == Some(2));
}

#[cfg(feature = "rand")]
#[test]
fn random_available_none_when_empty() {
    let mut s = RandomAvailable;
    assert_eq!(s.select(&[]), None);
}

// ── BestLeastRecentlyViewed ───────────────────────────────────────────────

#[test]
fn blrv_cycles_through_all_candidates() {
    let mut s = BestLeastRecentlyViewed::new();
    let cs = cand(&["a", "b", "c"]);
    assert_eq!(s.select(&cs), Some(0)); // a - unseen
    assert_eq!(s.select(&cs), Some(1)); // b - unseen
    assert_eq!(s.select(&cs), Some(2)); // c - unseen
    assert_eq!(s.select(&cs), Some(0)); // a - oldest seen
}

#[test]
fn blrv_skips_unavailable() {
    let mut s = BestLeastRecentlyViewed::new();
    let cs = cand_mask(&["a", "b", "c"], &[false, true, true]);
    assert_eq!(s.select(&cs), Some(1)); // b - first available
    assert_eq!(s.select(&cs), Some(2)); // c - next unseen
}

#[test]
fn blrv_none_when_all_unavailable() {
    let mut s = BestLeastRecentlyViewed::new();
    let cs = cand_mask(&["a", "b"], &[false, false]);
    assert_eq!(s.select(&cs), None);
}

#[test]
fn blrv_single_candidate_always_picks_it() {
    let mut s = BestLeastRecentlyViewed::new();
    let cs = cand(&["only"]);
    assert_eq!(s.select(&cs), Some(0));
    assert_eq!(s.select(&cs), Some(0));
}
