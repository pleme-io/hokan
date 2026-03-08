//! Hokan (補完) — unified completion engine for Neovim with LSP, buffer, path, and snippet sources
//!
//! Part of the blnvim-ng distribution — a Rust-native Neovim plugin suite.
//! Built with [`nvim-oxi`](https://github.com/noib3/nvim-oxi) for zero-cost
//! Neovim API bindings.

use nvim_oxi as oxi;

#[oxi::plugin]
fn hokan() -> oxi::Result<()> {
    Ok(())
}
