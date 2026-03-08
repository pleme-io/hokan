//! Simple snippet expansion engine.
//!
//! Supports `$1`, `$2`, ... tabstops and `${1:default}` placeholders.
//! This is a minimal engine — no nested placeholders, no transformations,
//! no mirroring.

/// A parsed tabstop within a snippet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tabstop {
    /// Tabstop number (0 = final cursor position, 1..n = navigation order).
    pub number: u32,
    /// Default text to insert at this tabstop. Empty if none.
    pub default_text: String,
    /// Byte offset in the expanded text where this tabstop starts.
    pub offset: usize,
    /// Length of the default text (0 for bare tabstops).
    pub len: usize,
}

/// Result of expanding a snippet template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpandedSnippet {
    /// The fully expanded text with tabstop markers removed and defaults inserted.
    pub text: String,
    /// Tabstops sorted by number (ascending). `$0` is always last for navigation.
    pub tabstops: Vec<Tabstop>,
}

/// Expand a snippet template string into text and tabstop positions.
///
/// Supported syntax:
/// - `$1`, `$2`, ... — bare tabstops (cursor positions)
/// - `$0` — final cursor position
/// - `${1:default}` — tabstop with default text
/// - `$$` — literal `$`
///
/// # Examples
///
/// ```
/// use hokan::snippet::expand;
///
/// let result = expand("fn ${1:name}($2) {\n    $0\n}");
/// assert_eq!(result.text, "fn name() {\n    \n}");
/// assert_eq!(result.tabstops.len(), 3);
/// ```
#[must_use]
pub fn expand(template: &str) -> ExpandedSnippet {
    let mut text = String::with_capacity(template.len());
    let mut tabstops = Vec::new();
    let chars: Vec<char> = template.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '$' {
            if i + 1 < len && chars[i + 1] == '$' {
                // Escaped dollar: $$
                text.push('$');
                i += 2;
            } else if i + 1 < len && chars[i + 1] == '{' {
                // ${N:default} or ${N}
                i += 2; // skip "${"
                let (tabstop, consumed) = parse_braced_tabstop(&chars[i..], text.len());
                if let Some(ts) = tabstop {
                    text.push_str(&ts.default_text);
                    tabstops.push(ts);
                    i += consumed;
                } else {
                    // Malformed — pass through literally
                    text.push_str("${");
                }
            } else if i + 1 < len && chars[i + 1].is_ascii_digit() {
                // $N bare tabstop
                i += 1; // skip '$'
                let (number, consumed) = parse_number(&chars[i..]);
                tabstops.push(Tabstop {
                    number,
                    default_text: String::new(),
                    offset: text.len(),
                    len: 0,
                });
                i += consumed;
            } else {
                // Lone $ at end or before non-digit — pass through
                text.push('$');
                i += 1;
            }
        } else {
            text.push(chars[i]);
            i += 1;
        }
    }

    // Sort: $1, $2, ... $N, then $0 last (final position).
    tabstops.sort_by_key(|t| if t.number == 0 { u32::MAX } else { t.number });

    ExpandedSnippet { text, tabstops }
}

/// Parse a number from the start of a char slice.
/// Returns `(number, chars_consumed)`.
fn parse_number(chars: &[char]) -> (u32, usize) {
    let mut n: u32 = 0;
    let mut consumed = 0;
    for &ch in chars {
        if ch.is_ascii_digit() {
            n = n.saturating_mul(10).saturating_add(u32::from(ch as u8 - b'0'));
            consumed += 1;
        } else {
            break;
        }
    }
    (n, consumed)
}

/// Parse a braced tabstop: `N:default}` or `N}`.
/// `chars` starts after `${`.
/// Returns `(Some(Tabstop), chars_consumed_including_closing_brace)` or `(None, 0)`.
fn parse_braced_tabstop(chars: &[char], current_offset: usize) -> (Option<Tabstop>, usize) {
    if chars.is_empty() {
        return (None, 0);
    }

    let (number, num_consumed) = parse_number(chars);
    if num_consumed == 0 {
        return (None, 0);
    }

    let rest = &chars[num_consumed..];
    if rest.is_empty() {
        return (None, 0);
    }

    if rest[0] == '}' {
        // ${N}
        return (
            Some(Tabstop {
                number,
                default_text: String::new(),
                offset: current_offset,
                len: 0,
            }),
            num_consumed + 1,
        );
    }

    if rest[0] == ':' {
        // ${N:default}
        // Find closing brace (no nesting support).
        let default_chars = &rest[1..];
        if let Some(close_pos) = default_chars.iter().position(|&c| c == '}') {
            let default_text: String = default_chars[..close_pos].iter().collect();
            let default_len = default_text.len();
            return (
                Some(Tabstop {
                    number,
                    default_text,
                    offset: current_offset,
                    len: default_len,
                }),
                num_consumed + 1 + close_pos + 1, // N + : + default + }
            );
        }
    }

    (None, 0)
}

/// Given an expanded snippet and a tabstop index (0-based into the sorted
/// tabstops vector), return the (offset, len) for the cursor/selection.
#[must_use]
pub fn tabstop_position(snippet: &ExpandedSnippet, index: usize) -> Option<(usize, usize)> {
    snippet.tabstops.get(index).map(|t| (t.offset, t.len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_no_tabstops() {
        let result = expand("hello world");
        assert_eq!(result.text, "hello world");
        assert!(result.tabstops.is_empty());
    }

    #[test]
    fn single_bare_tabstop() {
        let result = expand("hello $1 world");
        assert_eq!(result.text, "hello  world");
        assert_eq!(result.tabstops.len(), 1);
        assert_eq!(result.tabstops[0].number, 1);
        assert_eq!(result.tabstops[0].offset, 6);
        assert_eq!(result.tabstops[0].len, 0);
    }

    #[test]
    fn multiple_bare_tabstops() {
        let result = expand("$1 + $2 = $3");
        assert_eq!(result.text, " +  = ");
        assert_eq!(result.tabstops.len(), 3);
        assert_eq!(result.tabstops[0].number, 1);
        assert_eq!(result.tabstops[1].number, 2);
        assert_eq!(result.tabstops[2].number, 3);
    }

    #[test]
    fn tabstop_with_default() {
        let result = expand("fn ${1:name}() {}");
        assert_eq!(result.text, "fn name() {}");
        assert_eq!(result.tabstops.len(), 1);
        assert_eq!(result.tabstops[0].number, 1);
        assert_eq!(result.tabstops[0].default_text, "name");
        assert_eq!(result.tabstops[0].offset, 3);
        assert_eq!(result.tabstops[0].len, 4);
    }

    #[test]
    fn zero_tabstop_sorts_last() {
        let result = expand("$0 $2 $1");
        assert_eq!(result.tabstops.len(), 3);
        assert_eq!(result.tabstops[0].number, 1);
        assert_eq!(result.tabstops[1].number, 2);
        assert_eq!(result.tabstops[2].number, 0); // $0 last
    }

    #[test]
    fn escaped_dollar() {
        let result = expand("costs $$5");
        assert_eq!(result.text, "costs $5");
        assert!(result.tabstops.is_empty());
    }

    #[test]
    fn braced_tabstop_no_default() {
        let result = expand("hello ${1} world");
        assert_eq!(result.text, "hello  world");
        assert_eq!(result.tabstops.len(), 1);
        assert_eq!(result.tabstops[0].number, 1);
        assert_eq!(result.tabstops[0].offset, 6);
    }

    #[test]
    fn mixed_tabstops_and_defaults() {
        let result = expand("fn ${1:name}($2) {\n    $0\n}");
        assert_eq!(result.text, "fn name() {\n    \n}");
        assert_eq!(result.tabstops.len(), 3);
        // Sorted: $1, $2, $0
        assert_eq!(result.tabstops[0].number, 1);
        assert_eq!(result.tabstops[0].default_text, "name");
        assert_eq!(result.tabstops[1].number, 2);
        assert_eq!(result.tabstops[2].number, 0);
    }

    #[test]
    fn tabstop_position_returns_correct_values() {
        let result = expand("fn ${1:name}() {}");
        let pos = tabstop_position(&result, 0);
        assert_eq!(pos, Some((3, 4))); // "name" starts at offset 3, len 4
    }

    #[test]
    fn tabstop_position_out_of_bounds() {
        let result = expand("hello $1");
        assert_eq!(tabstop_position(&result, 5), None);
    }

    #[test]
    fn empty_template() {
        let result = expand("");
        assert_eq!(result.text, "");
        assert!(result.tabstops.is_empty());
    }

    #[test]
    fn lone_dollar_at_end() {
        let result = expand("hello$");
        assert_eq!(result.text, "hello$");
        assert!(result.tabstops.is_empty());
    }

    #[test]
    fn dollar_before_non_digit() {
        let result = expand("$a $b");
        assert_eq!(result.text, "$a $b");
        assert!(result.tabstops.is_empty());
    }

    #[test]
    fn multi_digit_tabstop() {
        let result = expand("$10 $11");
        assert_eq!(result.tabstops.len(), 2);
        assert_eq!(result.tabstops[0].number, 10);
        assert_eq!(result.tabstops[1].number, 11);
    }

    #[test]
    fn default_with_spaces() {
        let result = expand("${1:hello world}");
        assert_eq!(result.text, "hello world");
        assert_eq!(result.tabstops[0].default_text, "hello world");
        assert_eq!(result.tabstops[0].len, 11);
    }

    #[test]
    fn empty_default() {
        let result = expand("${1:}");
        assert_eq!(result.text, "");
        assert_eq!(result.tabstops[0].default_text, "");
        assert_eq!(result.tabstops[0].len, 0);
    }

    #[test]
    fn malformed_brace_passthrough() {
        // ${abc} — no number
        let result = expand("${abc}");
        assert_eq!(result.text, "${abc}");
        assert!(result.tabstops.is_empty());
    }

    #[test]
    fn unclosed_brace_passthrough() {
        let result = expand("${1:hello");
        assert_eq!(result.text, "${1:hello");
        assert!(result.tabstops.is_empty());
    }

    #[test]
    fn consecutive_tabstops() {
        let result = expand("$1$2$3");
        assert_eq!(result.text, "");
        assert_eq!(result.tabstops.len(), 3);
        // All at offset 0 since no text between them
        assert_eq!(result.tabstops[0].offset, 0);
        assert_eq!(result.tabstops[1].offset, 0);
        assert_eq!(result.tabstops[2].offset, 0);
    }

    #[test]
    fn realistic_rust_snippet() {
        let template = "impl ${1:Trait} for ${2:Type} {\n    $0\n}";
        let result = expand(template);
        assert_eq!(result.text, "impl Trait for Type {\n    \n}");
        assert_eq!(result.tabstops.len(), 3);
        assert_eq!(result.tabstops[0].number, 1);
        assert_eq!(result.tabstops[0].default_text, "Trait");
        assert_eq!(result.tabstops[1].number, 2);
        assert_eq!(result.tabstops[1].default_text, "Type");
        assert_eq!(result.tabstops[2].number, 0);
    }

    #[test]
    fn println_snippet() {
        let template = "println!(\"${1:{}}\", $2);$0";
        let result = expand(template);
        assert_eq!(result.text, "println!(\"{}\", );");
        assert_eq!(result.tabstops.len(), 3);
    }
}
