//! Completion item types and kinds.
//!
//! Represents a single completion candidate with label, kind,
//! detail text, insert text, and sort priority.

use std::fmt;

/// The kind of a completion item, matching LSP `CompletionItemKind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompletionKind {
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Unit,
    Value,
    Enum,
    Keyword,
    Snippet,
    Color,
    File,
    Reference,
    Folder,
    EnumMember,
    Constant,
    Struct,
    Event,
    Operator,
    TypeParameter,
}

impl CompletionKind {
    /// Icon string for display in the completion menu.
    ///
    /// Each kind maps to a distinct Nerd Font icon codepoint.
    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Text => "\u{f15c} ",          // nf-fa-file_text
            Self::Method => "\u{f6a6} ",         // nf-mdi-function_variant
            Self::Function => "\u{f0295} ",      // nf-md-function
            Self::Constructor => "\u{f0536} ",   // nf-md-code_braces
            Self::Field => "\u{f0374} ",         // nf-md-tag
            Self::Variable => "\u{f0ae7} ",      // nf-md-variable
            Self::Class => "\u{f0b22} ",         // nf-md-shape
            Self::Interface => "\u{f108} ",      // nf-fa-desktop (interface)
            Self::Module => "\u{f0d25} ",        // nf-md-package_variant
            Self::Property => "\u{f0ad4} ",      // nf-md-tune
            Self::Unit => "\u{f475} ",           // nf-oct-number
            Self::Value => "\u{f89f} ",          // nf-mdi-numeric
            Self::Enum => "\u{f0702} ",          // nf-md-format_list_numbered
            Self::Keyword => "\u{f0f4b} ",       // nf-md-key_variant
            Self::Snippet => "\u{f0e14} ",       // nf-md-text_box_outline
            Self::Color => "\u{f0266} ",         // nf-md-palette
            Self::File => "\u{f0214} ",          // nf-md-file
            Self::Reference => "\u{f0c95} ",     // nf-md-book_open_page_variant
            Self::Folder => "\u{f024b} ",        // nf-md-folder
            Self::EnumMember => "\u{f0703} ",    // nf-md-format_list_bulleted_type
            Self::Constant => "\u{f03a0} ",      // nf-md-pi (constant)
            Self::Struct => "\u{f0b22} ",        // nf-md-shape (same as class — both are types)
            Self::Event => "\u{f0adb} ",         // nf-md-flash
            Self::Operator => "\u{f0604} ",      // nf-md-math_compass
            Self::TypeParameter => "\u{f1507} ", // nf-md-alpha_t_box
        }
    }

    /// Short label for the kind.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Method => "Method",
            Self::Function => "Function",
            Self::Constructor => "Constructor",
            Self::Field => "Field",
            Self::Variable => "Variable",
            Self::Class => "Class",
            Self::Interface => "Interface",
            Self::Module => "Module",
            Self::Property => "Property",
            Self::Unit => "Unit",
            Self::Value => "Value",
            Self::Enum => "Enum",
            Self::Keyword => "Keyword",
            Self::Snippet => "Snippet",
            Self::Color => "Color",
            Self::File => "File",
            Self::Reference => "Reference",
            Self::Folder => "Folder",
            Self::EnumMember => "EnumMember",
            Self::Constant => "Constant",
            Self::Struct => "Struct",
            Self::Event => "Event",
            Self::Operator => "Operator",
            Self::TypeParameter => "TypeParam",
        }
    }

    /// Sort priority (lower = higher priority in menu).
    #[must_use]
    pub const fn sort_priority(self) -> u8 {
        match self {
            Self::Variable | Self::Field | Self::Property => 0,
            Self::Function | Self::Method | Self::Constructor => 1,
            Self::Keyword => 2,
            Self::Snippet => 3,
            Self::Module | Self::Class | Self::Struct | Self::Interface | Self::Enum => 4,
            Self::Constant | Self::EnumMember | Self::Value | Self::Unit => 5,
            Self::File | Self::Folder => 6,
            Self::Text => 7,
            Self::Color
            | Self::Reference
            | Self::Event
            | Self::Operator
            | Self::TypeParameter => 8,
        }
    }
}

impl fmt::Display for CompletionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// The source that produced a completion item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompletionSource {
    Lsp,
    Buffer,
    Path,
    Snippet,
}

impl CompletionSource {
    /// Short tag for display.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Lsp => "[LSP]",
            Self::Buffer => "[Buf]",
            Self::Path => "[Path]",
            Self::Snippet => "[Snip]",
        }
    }
}

impl fmt::Display for CompletionSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.tag())
    }
}

/// A single completion candidate.
#[derive(Debug, Clone)]
pub struct CompletionItem {
    /// The display label shown in the completion menu.
    pub label: String,
    /// The kind of this completion item.
    pub kind: CompletionKind,
    /// Optional detail text (type signature, file path, etc.).
    pub detail: Option<String>,
    /// Text to insert when the item is confirmed.
    /// If `None`, `label` is used.
    pub insert_text: Option<String>,
    /// Sort key override. If `None`, derived from kind priority + label.
    pub sort_text: Option<String>,
    /// Filter text for fuzzy matching. If `None`, `label` is used.
    pub filter_text: Option<String>,
    /// Which source produced this item.
    pub source: CompletionSource,
}

impl CompletionItem {
    /// Create a new completion item with the given label, kind, and source.
    #[must_use]
    pub fn new(label: &str, kind: CompletionKind, source: CompletionSource) -> Self {
        Self {
            label: label.to_string(),
            kind,
            detail: None,
            insert_text: None,
            sort_text: None,
            filter_text: None,
            source,
        }
    }

    /// Set detail text.
    #[must_use]
    pub fn detail(mut self, detail: &str) -> Self {
        self.detail = Some(detail.to_string());
        self
    }

    /// Set insert text (what gets inserted on confirm).
    #[must_use]
    pub fn insert_text(mut self, text: &str) -> Self {
        self.insert_text = Some(text.to_string());
        self
    }

    /// Set sort text override.
    #[must_use]
    pub fn sort_text(mut self, text: &str) -> Self {
        self.sort_text = Some(text.to_string());
        self
    }

    /// Set filter text for fuzzy matching.
    #[must_use]
    pub fn filter_text(mut self, text: &str) -> Self {
        self.filter_text = Some(text.to_string());
        self
    }

    /// The text used for inserting (`insert_text` or label).
    #[must_use]
    pub fn text_to_insert(&self) -> &str {
        self.insert_text.as_deref().unwrap_or(&self.label)
    }

    /// The text used for filtering (`filter_text` or label).
    #[must_use]
    pub fn text_to_filter(&self) -> &str {
        self.filter_text.as_deref().unwrap_or(&self.label)
    }

    /// The text used for sorting.
    /// Format: `{priority}{sort_text_or_label}` for stable ordering.
    #[must_use]
    pub fn text_to_sort(&self) -> String {
        let priority = self.kind.sort_priority();
        let base = self
            .sort_text
            .as_deref()
            .unwrap_or(&self.label);
        format!("{priority}{base}")
    }

    /// Render a single-line display string: "icon label  detail  [source]"
    #[must_use]
    pub fn render_line(&self, max_label_width: usize) -> String {
        let icon = self.kind.icon();
        let padded_label = format!("{:<width$}", self.label, width = max_label_width);
        match &self.detail {
            Some(d) => format!("{icon}{padded_label}  {d}  {}", self.source.tag()),
            None => format!("{icon}{padded_label}  {}", self.source.tag()),
        }
    }
}

/// Sort a slice of completion items by their sort key.
pub fn sort_items(items: &mut [CompletionItem]) {
    items.sort_by_key(CompletionItem::text_to_sort);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_icon_is_non_empty() {
        let kinds = [
            CompletionKind::Text,
            CompletionKind::Method,
            CompletionKind::Function,
            CompletionKind::Snippet,
            CompletionKind::File,
            CompletionKind::Folder,
            CompletionKind::Keyword,
        ];
        for kind in kinds {
            assert!(!kind.icon().is_empty(), "{kind:?} has empty icon");
        }
    }

    #[test]
    fn kind_sort_priority_ordering() {
        // Variables/fields should sort before text
        assert!(CompletionKind::Variable.sort_priority() < CompletionKind::Text.sort_priority());
        // Functions before keywords
        assert!(CompletionKind::Function.sort_priority() < CompletionKind::Keyword.sort_priority());
        // Snippets before files
        assert!(CompletionKind::Snippet.sort_priority() < CompletionKind::File.sort_priority());
    }

    #[test]
    fn item_text_to_insert_defaults_to_label() {
        let item = CompletionItem::new("hello", CompletionKind::Text, CompletionSource::Buffer);
        assert_eq!(item.text_to_insert(), "hello");
    }

    #[test]
    fn item_text_to_insert_uses_override() {
        let item = CompletionItem::new("println!", CompletionKind::Snippet, CompletionSource::Snippet)
            .insert_text("println!(\"$1\")$0");
        assert_eq!(item.text_to_insert(), "println!(\"$1\")$0");
    }

    #[test]
    fn item_text_to_filter_defaults_to_label() {
        let item = CompletionItem::new("foo", CompletionKind::Function, CompletionSource::Lsp);
        assert_eq!(item.text_to_filter(), "foo");
    }

    #[test]
    fn item_text_to_filter_uses_override() {
        let item = CompletionItem::new("foo()", CompletionKind::Function, CompletionSource::Lsp)
            .filter_text("foo");
        assert_eq!(item.text_to_filter(), "foo");
    }

    #[test]
    fn item_sort_text_includes_priority() {
        let var_item =
            CompletionItem::new("alpha", CompletionKind::Variable, CompletionSource::Lsp);
        let text_item =
            CompletionItem::new("alpha", CompletionKind::Text, CompletionSource::Buffer);
        // Variable (priority 0) should sort before Text (priority 7)
        assert!(var_item.text_to_sort() < text_item.text_to_sort());
    }

    #[test]
    fn sort_items_by_kind_then_label() {
        let mut items = vec![
            CompletionItem::new("zebra", CompletionKind::Text, CompletionSource::Buffer),
            CompletionItem::new("alpha", CompletionKind::Variable, CompletionSource::Lsp),
            CompletionItem::new("beta", CompletionKind::Function, CompletionSource::Lsp),
        ];
        sort_items(&mut items);
        assert_eq!(items[0].label, "alpha"); // Variable, priority 0
        assert_eq!(items[1].label, "beta"); // Function, priority 1
        assert_eq!(items[2].label, "zebra"); // Text, priority 7
    }

    #[test]
    fn sort_items_same_kind_by_label() {
        let mut items = vec![
            CompletionItem::new("gamma", CompletionKind::Function, CompletionSource::Lsp),
            CompletionItem::new("alpha", CompletionKind::Function, CompletionSource::Lsp),
            CompletionItem::new("beta", CompletionKind::Function, CompletionSource::Lsp),
        ];
        sort_items(&mut items);
        assert_eq!(items[0].label, "alpha");
        assert_eq!(items[1].label, "beta");
        assert_eq!(items[2].label, "gamma");
    }

    #[test]
    fn render_line_without_detail() {
        let item = CompletionItem::new("foo", CompletionKind::Function, CompletionSource::Lsp);
        let line = item.render_line(10);
        assert!(line.contains("foo"));
        assert!(line.contains("[LSP]"));
    }

    #[test]
    fn render_line_with_detail() {
        let item = CompletionItem::new("foo", CompletionKind::Function, CompletionSource::Lsp)
            .detail("fn()");
        let line = item.render_line(10);
        assert!(line.contains("foo"));
        assert!(line.contains("fn()"));
        assert!(line.contains("[LSP]"));
    }

    #[test]
    fn source_tag() {
        assert_eq!(CompletionSource::Lsp.tag(), "[LSP]");
        assert_eq!(CompletionSource::Buffer.tag(), "[Buf]");
        assert_eq!(CompletionSource::Path.tag(), "[Path]");
        assert_eq!(CompletionSource::Snippet.tag(), "[Snip]");
    }

    #[test]
    fn sort_text_override() {
        let mut items = vec![
            CompletionItem::new("beta", CompletionKind::Function, CompletionSource::Lsp)
                .sort_text("aaa"),
            CompletionItem::new("alpha", CompletionKind::Function, CompletionSource::Lsp)
                .sort_text("zzz"),
        ];
        sort_items(&mut items);
        // "beta" has sort_text "aaa" so it sorts first despite label
        assert_eq!(items[0].label, "beta");
        assert_eq!(items[1].label, "alpha");
    }
}
