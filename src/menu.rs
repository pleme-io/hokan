//! Completion popup menu rendering and navigation.
//!
//! Uses waku's `FloatWindow` and `ListState` for the UI, and furui's
//! `FuzzyMatcher` for filtering candidates by typed prefix.

use crate::item::CompletionItem;
use furui::FuzzyMatcher;
use waku::border::BorderStyle;
use waku::float::FloatWindow;
use waku::layout::{Anchor, FloatLayout, Size};
use waku::list::{ListItem, ListState};

/// Maximum number of visible items in the completion menu.
const MAX_VISIBLE_ITEMS: usize = 10;

/// Maximum number of total candidates to consider.
const MAX_CANDIDATES: usize = 200;

/// The completion menu state and window.
pub struct CompletionMenu {
    /// The floating window backing the popup.
    window: FloatWindow,
    /// Current list navigation state.
    list: ListState,
    /// All unfiltered completion items.
    candidates: Vec<CompletionItem>,
    /// Indices into `candidates` that pass the current filter.
    filtered_indices: Vec<usize>,
    /// The current filter/prefix text.
    filter: String,
    /// Fuzzy matcher for filtering.
    matcher: FuzzyMatcher,
}

impl CompletionMenu {
    /// Create a new completion menu (not yet visible).
    #[must_use]
    pub fn new() -> Self {
        Self {
            window: FloatWindow::new()
                .layout(FloatLayout {
                    width: Size::Fixed(50),
                    #[allow(clippy::cast_possible_truncation)]
                    height: Size::Fixed(MAX_VISIBLE_ITEMS as u32),
                    anchor: Anchor::NorthWest,
                    row_offset: 1,
                    col_offset: 0,
                })
                .border(BorderStyle::Rounded)
                .focusable(false),
            list: ListState::new(Vec::new(), MAX_VISIBLE_ITEMS),
            candidates: Vec::new(),
            filtered_indices: Vec::new(),
            filter: String::new(),
            matcher: FuzzyMatcher::new(),
        }
    }

    /// Set the completion candidates and filter prefix, then update the display.
    pub fn set_candidates(
        &mut self,
        mut candidates: Vec<CompletionItem>,
        prefix: &str,
    ) {
        candidates.truncate(MAX_CANDIDATES);
        self.candidates = candidates;
        self.filter = prefix.to_string();
        self.refilter();
    }

    /// Update the filter prefix (e.g., as the user types more characters).
    pub fn update_filter(&mut self, prefix: &str) {
        self.filter = prefix.to_string();
        self.refilter();
    }

    /// Recompute filtered indices from current candidates + filter.
    fn refilter(&mut self) {
        if self.filter.is_empty() {
            self.filtered_indices = (0..self.candidates.len()).collect();
        } else {
            let filter_texts: Vec<&str> = self
                .candidates
                .iter()
                .map(CompletionItem::text_to_filter)
                .collect();

            let ranked = self.matcher.rank(&self.filter, &filter_texts);
            self.filtered_indices = ranked.iter().map(|r| r.index).collect();
        }

        // Rebuild the list items for display.
        let max_label = self
            .filtered_indices
            .iter()
            .map(|&i| self.candidates[i].label.len())
            .max()
            .unwrap_or(0)
            .min(40);

        let list_items: Vec<ListItem> = self
            .filtered_indices
            .iter()
            .map(|&i| {
                let ci = &self.candidates[i];
                let display = ci.render_line(max_label);
                ListItem::new(&display).data(&i.to_string())
            })
            .collect();

        self.list = ListState::new(list_items, MAX_VISIBLE_ITEMS);
    }

    /// Show the completion popup. Opens the floating window.
    pub fn show(&mut self) -> tane::Result<()> {
        if self.filtered_indices.is_empty() {
            return self.hide();
        }
        if !self.window.is_open() {
            self.window.open()?;
        }
        self.update_window_content()
    }

    /// Hide the completion popup.
    pub fn hide(&mut self) -> tane::Result<()> {
        self.window.close()?;
        self.candidates.clear();
        self.filtered_indices.clear();
        self.filter.clear();
        Ok(())
    }

    /// Select the next item.
    pub fn select_next(&mut self) -> tane::Result<()> {
        self.list.select_next();
        self.update_window_content()
    }

    /// Select the previous item.
    pub fn select_prev(&mut self) -> tane::Result<()> {
        self.list.select_prev();
        self.update_window_content()
    }

    /// Confirm the currently selected item.
    /// Returns the `CompletionItem` to insert, or `None` if nothing is selected.
    #[must_use]
    pub fn confirm(&self) -> Option<&CompletionItem> {
        let list_item = self.list.selected_item()?;
        let idx: usize = list_item.data.as_ref()?.parse().ok()?;
        self.candidates.get(idx)
    }

    /// Whether the menu is currently visible.
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.window.is_open() && !self.filtered_indices.is_empty()
    }

    /// Number of filtered items.
    #[must_use]
    pub fn item_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Currently selected index in the filtered list.
    #[must_use]
    pub fn selected_index(&self) -> usize {
        self.list.selected
    }

    /// Push rendered lines to the floating window buffer.
    fn update_window_content(&mut self) -> tane::Result<()> {
        let lines = self.list.render_lines();
        let line_refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        self.window.set_lines(&line_refs)?;
        Ok(())
    }
}

impl Default for CompletionMenu {
    fn default() -> Self {
        Self::new()
    }
}
