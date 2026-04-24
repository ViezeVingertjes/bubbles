//! Rolling log of everything the dialogue runner has emitted this session.

use bubbles::MarkupSpan;

/// A single entry in the running session log.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TranscriptEntry {
    /// A node has started executing.
    NodeStarted(String),
    /// A node has finished.
    NodeComplete(String),
    /// A line of dialogue was surfaced.
    Line {
        /// Optional `Speaker:` prefix.
        speaker: Option<String>,
        /// Fully interpolated line text.
        text: String,
        /// Markup spans for styled regions of `text`.
        spans: Vec<MarkupSpan>,
    },
    /// A host command was emitted.
    Command {
        /// Command name.
        name: String,
        /// Already-interpolated args.
        args: Vec<String>,
        /// Trailing tag metadata.
        tags: Vec<String>,
    },
    /// The player picked an option.
    OptionChosen {
        /// The option's display text.
        text: String,
        /// The 0-based index in the option set.
        index: usize,
    },
}

/// Bounded append-only transcript with a separately tracked view offset.
///
/// Entries older than [`Transcript::CAPACITY`] are dropped from the front so
/// the memory footprint stays bounded regardless of how long a dialogue
/// session runs.
pub struct Transcript {
    entries: Vec<TranscriptEntry>,
    /// How many entries back from the newest the view is scrolled.
    /// `0` means "newest entry visible at the bottom".
    scroll: usize,
}

impl Transcript {
    /// Maximum number of entries retained.
    pub const CAPACITY: usize = 512;

    pub(crate) const fn new() -> Self {
        Self {
            entries: Vec::new(),
            scroll: 0,
        }
    }

    pub(crate) fn push(&mut self, entry: TranscriptEntry) {
        if self.entries.len() == Self::CAPACITY {
            self.entries.remove(0);
        }
        self.entries.push(entry);
        // Pushing a new entry while the view is scrolled back shouldn't
        // yank the viewport to the tail - preserve the offset, but clamp
        // it so it still refers to a real entry.
        let max = self.entries.len().saturating_sub(1);
        if self.scroll > max {
            self.scroll = max;
        }
    }

    pub(crate) fn as_slice(&self) -> &[TranscriptEntry] {
        &self.entries
    }

    pub(crate) const fn scroll(&self) -> usize {
        self.scroll
    }

    pub(crate) const fn scroll_up(&mut self) {
        let max = self.entries.len().saturating_sub(1);
        if self.scroll < max {
            self.scroll += 1;
        }
    }

    pub(crate) const fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }
}
