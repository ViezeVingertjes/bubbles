//! Geometry shared by the popups drawn over the dialogue panes.

use ratatui::layout::Rect;

/// Returns a rectangle with `width_pct`% × `height_pct`% of `area`, centered.
pub const fn centered(area: Rect, width_pct: u16, height_pct: u16) -> Rect {
    let width = area.width.saturating_mul(width_pct) / 100;
    let height = area.height.saturating_mul(height_pct) / 100;
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width,
        height,
    }
}
