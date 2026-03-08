//! Auto-pair logic for brackets and quotes.
//!
//! Pure Rust module that determines what text transformations to apply
//! when the user types an opening bracket, closing bracket, quote, or
//! backspace. All functions are side-effect free and testable without
//! Neovim.

/// A paired delimiter definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pair {
    pub open: char,
    pub close: char,
}

/// The default set of auto-pairs.
pub const DEFAULT_PAIRS: &[Pair] = &[
    Pair { open: '(', close: ')' },
    Pair { open: '[', close: ']' },
    Pair { open: '{', close: '}' },
    Pair { open: '"', close: '"' },
    Pair { open: '\'', close: '\'' },
    Pair { open: '`', close: '`' },
];

/// What action to take after typing a character.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairAction {
    /// Insert both the open and close character, cursor between them.
    InsertPair { open: char, close: char },
    /// Skip over an existing closing character (cursor moves right).
    SkipClose { close: char },
    /// Just insert the character as-is (no pair action).
    Passthrough,
}

/// What action to take on backspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackspaceAction {
    /// Delete both the opening and closing character of a pair.
    DeletePair,
    /// Normal single-character backspace.
    Normal,
}

/// Determine the action for a typed character given surrounding context.
///
/// `char_before`: the character immediately before the cursor (None at line start).
/// `char_after`: the character immediately after the cursor (None at line end).
/// `typed`: the character being typed.
#[must_use]
pub fn on_char_typed(
    char_before: Option<char>,
    char_after: Option<char>,
    typed: char,
    pairs: &[Pair],
) -> PairAction {
    // Check if typed is a closing char that matches what's after cursor.
    if let Some(after) = char_after
        && typed == after
        && is_close_char(typed, pairs)
    {
        return PairAction::SkipClose { close: typed };
    }

    // Check if typed is an opening char.
    if let Some(pair) = find_pair_by_open(typed, pairs) {
        // For quote-like pairs (open == close), extra checks:
        if pair.open == pair.close {
            // Don't auto-pair if char_before is alphanumeric (likely mid-word).
            if char_before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                return PairAction::Passthrough;
            }
        }
        // Don't auto-pair if char_after is alphanumeric (completing into a word).
        if char_after.is_some_and(|c| c.is_alphanumeric() || c == '_') {
            return PairAction::Passthrough;
        }
        return PairAction::InsertPair {
            open: pair.open,
            close: pair.close,
        };
    }

    PairAction::Passthrough
}

/// Determine the backspace action given the characters around the cursor.
///
/// `char_before`: the character immediately before the cursor.
/// `char_after`: the character immediately after the cursor.
#[must_use]
pub fn on_backspace(
    char_before: Option<char>,
    char_after: Option<char>,
    pairs: &[Pair],
) -> BackspaceAction {
    match (char_before, char_after) {
        (Some(before), Some(after)) => {
            if pairs.iter().any(|p| p.open == before && p.close == after) {
                BackspaceAction::DeletePair
            } else {
                BackspaceAction::Normal
            }
        }
        _ => BackspaceAction::Normal,
    }
}

/// Find a pair definition by its opening character.
fn find_pair_by_open(ch: char, pairs: &[Pair]) -> Option<&Pair> {
    pairs.iter().find(|p| p.open == ch)
}

/// Check if a character is a closing character in any pair.
fn is_close_char(ch: char, pairs: &[Pair]) -> bool {
    pairs.iter().any(|p| p.close == ch)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- InsertPair tests ----

    #[test]
    fn open_paren_inserts_pair() {
        let action = on_char_typed(None, None, '(', DEFAULT_PAIRS);
        assert_eq!(
            action,
            PairAction::InsertPair {
                open: '(',
                close: ')'
            }
        );
    }

    #[test]
    fn open_bracket_inserts_pair() {
        let action = on_char_typed(None, None, '[', DEFAULT_PAIRS);
        assert_eq!(
            action,
            PairAction::InsertPair {
                open: '[',
                close: ']'
            }
        );
    }

    #[test]
    fn open_brace_inserts_pair() {
        let action = on_char_typed(None, None, '{', DEFAULT_PAIRS);
        assert_eq!(
            action,
            PairAction::InsertPair {
                open: '{',
                close: '}'
            }
        );
    }

    #[test]
    fn double_quote_inserts_pair() {
        let action = on_char_typed(None, None, '"', DEFAULT_PAIRS);
        assert_eq!(
            action,
            PairAction::InsertPair {
                open: '"',
                close: '"'
            }
        );
    }

    #[test]
    fn single_quote_inserts_pair() {
        let action = on_char_typed(None, None, '\'', DEFAULT_PAIRS);
        assert_eq!(
            action,
            PairAction::InsertPair {
                open: '\'',
                close: '\''
            }
        );
    }

    #[test]
    fn backtick_inserts_pair() {
        let action = on_char_typed(None, None, '`', DEFAULT_PAIRS);
        assert_eq!(
            action,
            PairAction::InsertPair {
                open: '`',
                close: '`'
            }
        );
    }

    // ---- SkipClose tests ----

    #[test]
    fn close_paren_skips_when_after_cursor() {
        let action = on_char_typed(Some('x'), Some(')'), ')', DEFAULT_PAIRS);
        assert_eq!(action, PairAction::SkipClose { close: ')' });
    }

    #[test]
    fn close_bracket_skips() {
        let action = on_char_typed(Some('x'), Some(']'), ']', DEFAULT_PAIRS);
        assert_eq!(action, PairAction::SkipClose { close: ']' });
    }

    #[test]
    fn close_brace_skips() {
        let action = on_char_typed(Some('x'), Some('}'), '}', DEFAULT_PAIRS);
        assert_eq!(action, PairAction::SkipClose { close: '}' });
    }

    #[test]
    fn quote_skips_when_after_cursor() {
        // Typing " when " is already the next char => skip
        let action = on_char_typed(Some('x'), Some('"'), '"', DEFAULT_PAIRS);
        assert_eq!(action, PairAction::SkipClose { close: '"' });
    }

    // ---- Passthrough tests ----

    #[test]
    fn regular_char_is_passthrough() {
        let action = on_char_typed(None, None, 'a', DEFAULT_PAIRS);
        assert_eq!(action, PairAction::Passthrough);
    }

    #[test]
    fn quote_after_alphanumeric_is_passthrough() {
        // e.g., typing ' in "it's" -> don't auto-pair
        let action = on_char_typed(Some('t'), None, '\'', DEFAULT_PAIRS);
        assert_eq!(action, PairAction::Passthrough);
    }

    #[test]
    fn quote_after_underscore_is_passthrough() {
        let action = on_char_typed(Some('_'), None, '"', DEFAULT_PAIRS);
        assert_eq!(action, PairAction::Passthrough);
    }

    #[test]
    fn open_paren_before_alphanumeric_is_passthrough() {
        // Don't auto-pair if the next char is a letter (completing into word)
        let action = on_char_typed(None, Some('a'), '(', DEFAULT_PAIRS);
        assert_eq!(action, PairAction::Passthrough);
    }

    #[test]
    fn open_paren_before_space_inserts_pair() {
        let action = on_char_typed(Some('f'), Some(' '), '(', DEFAULT_PAIRS);
        assert_eq!(
            action,
            PairAction::InsertPair {
                open: '(',
                close: ')'
            }
        );
    }

    // ---- Backspace tests ----

    #[test]
    fn backspace_deletes_pair() {
        let action = on_backspace(Some('('), Some(')'), DEFAULT_PAIRS);
        assert_eq!(action, BackspaceAction::DeletePair);
    }

    #[test]
    fn backspace_deletes_bracket_pair() {
        let action = on_backspace(Some('['), Some(']'), DEFAULT_PAIRS);
        assert_eq!(action, BackspaceAction::DeletePair);
    }

    #[test]
    fn backspace_deletes_brace_pair() {
        let action = on_backspace(Some('{'), Some('}'), DEFAULT_PAIRS);
        assert_eq!(action, BackspaceAction::DeletePair);
    }

    #[test]
    fn backspace_deletes_quote_pair() {
        let action = on_backspace(Some('"'), Some('"'), DEFAULT_PAIRS);
        assert_eq!(action, BackspaceAction::DeletePair);
    }

    #[test]
    fn backspace_normal_when_no_pair() {
        let action = on_backspace(Some('a'), Some('b'), DEFAULT_PAIRS);
        assert_eq!(action, BackspaceAction::Normal);
    }

    #[test]
    fn backspace_normal_at_line_start() {
        let action = on_backspace(None, Some(')'), DEFAULT_PAIRS);
        assert_eq!(action, BackspaceAction::Normal);
    }

    #[test]
    fn backspace_normal_at_line_end() {
        let action = on_backspace(Some('('), None, DEFAULT_PAIRS);
        assert_eq!(action, BackspaceAction::Normal);
    }

    #[test]
    fn backspace_mismatched_pair_is_normal() {
        let action = on_backspace(Some('('), Some(']'), DEFAULT_PAIRS);
        assert_eq!(action, BackspaceAction::Normal);
    }

    // ---- Edge cases ----

    #[test]
    fn empty_pairs_always_passthrough() {
        let action = on_char_typed(None, None, '(', &[]);
        assert_eq!(action, PairAction::Passthrough);
    }

    #[test]
    fn empty_pairs_backspace_always_normal() {
        let action = on_backspace(Some('('), Some(')'), &[]);
        assert_eq!(action, BackspaceAction::Normal);
    }

    #[test]
    fn open_brace_at_end_of_line() {
        let action = on_char_typed(Some(' '), None, '{', DEFAULT_PAIRS);
        assert_eq!(
            action,
            PairAction::InsertPair {
                open: '{',
                close: '}'
            }
        );
    }

    #[test]
    fn quote_at_line_start() {
        let action = on_char_typed(None, Some(' '), '"', DEFAULT_PAIRS);
        assert_eq!(
            action,
            PairAction::InsertPair {
                open: '"',
                close: '"'
            }
        );
    }

    #[test]
    fn paren_before_closing_bracket_inserts_pair() {
        // e.g., fn(|) where | is cursor, typing ( before )
        let action = on_char_typed(Some('n'), Some(')'), '(', DEFAULT_PAIRS);
        assert_eq!(
            action,
            PairAction::InsertPair {
                open: '(',
                close: ')'
            }
        );
    }
}
