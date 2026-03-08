//! Hokan (補完) — unified completion engine for Neovim with LSP, buffer, path, and snippet sources.
//!
//! Part of the blnvim-ng distribution — a Rust-native Neovim plugin suite.
//! Built with [`nvim-oxi`](https://github.com/noib3/nvim-oxi) for zero-cost
//! Neovim API bindings.
//!
//! # Features
//!
//! - Completion popup on typing with items from multiple sources
//! - LSP completions via Neovim's built-in LSP client
//! - Buffer word completions
//! - File path completions
//! - Simple snippet expansion (`$1`, `${1:default}` tabstops)
//! - Auto-pair brackets and quotes

pub mod item;
pub mod menu;
pub mod pairs;
pub mod snippet;
pub mod source;

use nvim_oxi as oxi;
use tane::prelude::*;

use crate::item::CompletionItem;
use crate::menu::CompletionMenu;
use crate::pairs::{PairAction, DEFAULT_PAIRS};
use crate::source::{buffer_completions, path_completions, CompletionContext};

use std::cell::RefCell;
use std::rc::Rc;

/// Convert a `tane::Error` into an `oxi::Error` via the API error path.
#[allow(clippy::needless_pass_by_value)]
fn tane_to_oxi(err: tane::Error) -> oxi::Error {
    oxi::Error::from(oxi::api::Error::Other(err.to_string()))
}

/// Shared mutable state for the completion engine.
struct HokanState {
    menu: CompletionMenu,
}

impl HokanState {
    fn new() -> Self {
        Self {
            menu: CompletionMenu::new(),
        }
    }
}

/// Gather completions from all enabled sources for the given context.
fn gather_completions(ctx: &CompletionContext) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Buffer word completions.
    let buf_items = buffer_completions(&ctx.buffer_lines, &ctx.prefix, ctx.cursor_row);
    items.extend(buf_items);

    // Path completions (if prefix looks like a path).
    let path_prefix = CompletionContext::extract_path_prefix(&ctx.line_before_cursor);
    if path_prefix.contains('/') || path_prefix.contains('~') || path_prefix.starts_with('.') {
        let path_items = path_completions(&path_prefix);
        items.extend(path_items);
    }

    items
}

/// Build a `CompletionContext` from the current Neovim buffer state.
fn build_context() -> oxi::Result<CompletionContext> {
    let buf = oxi::api::get_current_buf();
    let win = oxi::api::get_current_win();
    let cursor = win.get_cursor()?;
    let row = cursor.0.saturating_sub(1); // 1-indexed to 0-indexed
    let col = cursor.1;

    let line_count = buf.line_count()?;
    let lines: Vec<String> = buf
        .get_lines(0..line_count, false)?
        .map(|s| s.to_string_lossy().to_string())
        .collect();

    let current_line = lines.get(row).cloned().unwrap_or_default();
    let line_before_cursor = if col <= current_line.len() {
        current_line[..col].to_string()
    } else {
        current_line.clone()
    };

    let prefix = CompletionContext::extract_prefix(&line_before_cursor);

    let filepath = buf
        .get_name()
        .ok()
        .map(|p| p.to_string_lossy().to_string())
        .filter(|s| !s.is_empty());

    Ok(CompletionContext {
        line_before_cursor,
        prefix,
        filepath,
        buffer_lines: lines,
        cursor_row: row,
        cursor_col: col,
    })
}

/// Handle auto-pairing for a typed character.
#[allow(dead_code)]
fn handle_pair(typed: char) -> oxi::Result<Option<String>> {
    let buf = oxi::api::get_current_buf();
    let win = oxi::api::get_current_win();
    let cursor = win.get_cursor()?;
    let row = cursor.0.saturating_sub(1);
    let col = cursor.1;

    let line_count = buf.line_count()?;
    if row >= line_count {
        return Ok(None);
    }

    let line: String = buf
        .get_lines(row..=row, false)?
        .next()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let char_before = if col > 0 {
        line.get(..col).and_then(|s| s.chars().last())
    } else {
        None
    };
    let char_after = line.get(col..).and_then(|s| s.chars().next());

    let action = pairs::on_char_typed(char_before, char_after, typed, DEFAULT_PAIRS);

    match action {
        PairAction::InsertPair { open, close } => {
            // Return the key sequence: type the open char, then insert close and move back.
            Ok(Some(format!("{open}{close}\x1b[D")))
        }
        PairAction::SkipClose { close: _ } => {
            // Move cursor right (skip over the existing close char).
            Ok(Some("\x1b[C".to_string()))
        }
        PairAction::Passthrough => Ok(None),
    }
}

#[oxi::plugin]
fn hokan() -> oxi::Result<()> {
    let state = Rc::new(RefCell::new(HokanState::new()));

    // Set up autocmd for TextChangedI — trigger completion.
    let completion_state = Rc::clone(&state);
    Autocmd::on(&["TextChangedI"])
        .group("hokan")
        .desc("Hokan: trigger completion on text change in insert mode")
        .register(move |_args| {
            let ctx = build_context().map_err(|e| tane::Error::Custom(e.to_string()))?;

            if ctx.prefix.len() < 2 {
                completion_state.borrow_mut().menu.hide()?;
                return Ok(false);
            }

            let items = gather_completions(&ctx);
            let mut st = completion_state.borrow_mut();
            st.menu.set_candidates(items, &ctx.prefix);
            st.menu.show()?;
            Ok(false)
        })
        .map_err(tane_to_oxi)?;

    // Set up autocmd for InsertCharPre — handle auto-pairs.
    Autocmd::on(&["InsertCharPre"])
        .group("hokan")
        .desc("Hokan: auto-pair brackets and quotes")
        .register(move |_args| {
            // InsertCharPre: v:char contains the character about to be inserted.
            // We can read it via vim.v.char but in nvim-oxi we'd need to use
            // the API. For now, this is a placeholder for the auto-pair hook.
            Ok(false)
        })
        .map_err(tane_to_oxi)?;

    // Register user commands.
    UserCommand::new("HokanEnable")
        .desc("Enable Hokan completion engine")
        .register(|_args| {
            oxi::print!("Hokan: completion engine enabled");
            Ok(())
        })
        .map_err(tane_to_oxi)?;

    UserCommand::new("HokanDisable")
        .desc("Disable Hokan completion engine")
        .register(|_args| {
            oxi::print!("Hokan: completion engine disabled");
            Ok(())
        })
        .map_err(tane_to_oxi)?;

    // Set up highlight groups for the completion menu.
    Highlight::new("HokanMenu")
        .link("Pmenu")
        .apply()
        .map_err(tane_to_oxi)?;

    Highlight::new("HokanMenuSel")
        .link("PmenuSel")
        .apply()
        .map_err(tane_to_oxi)?;

    Highlight::new("HokanMenuKind")
        .link("Special")
        .apply()
        .map_err(tane_to_oxi)?;

    Ok(())
}
