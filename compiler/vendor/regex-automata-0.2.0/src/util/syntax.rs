use regex_syntax::ParserBuilder;

/// A common set of configuration options that apply to the syntax of a regex.
///
/// This represents a group of configuration options that specifically apply
/// to how the concrete syntax of a regular expression is interpreted. In
/// particular, they are generally forwarded to the
/// [`ParserBuilder`](https://docs.rs/regex-syntax/*/regex_syntax/struct.ParserBuilder.html)
/// in the
/// [`regex-syntax`](https://docs.rs/regex-syntax)
/// crate when building a regex from its concrete syntax directly.
///
/// These options are defined as a group since they apply to every regex engine
/// in this crate. Instead of re-defining them on every engine's builder, they
/// are instead provided here as one cohesive unit.
#[derive(Clone, Copy, Debug)]
pub struct SyntaxConfig {
    case_insensitive: bool,
    multi_line: bool,
    dot_matches_new_line: bool,
    swap_greed: bool,
    ignore_whitespace: bool,
    unicode: bool,
    utf8: bool,
    nest_limit: u32,
    octal: bool,
}

impl SyntaxConfig {
    /// Return a new default syntax configuration.
    pub fn new() -> SyntaxConfig {
        // These defaults match the ones used in regex-syntax.
        SyntaxConfig {
            case_insensitive: false,
            multi_line: false,
            dot_matches_new_line: false,
            swap_greed: false,
            ignore_whitespace: false,
            unicode: true,
            utf8: true,
            nest_limit: 250,
            octal: false,
        }
    }

    /// Enable or disable the case insensitive flag by default.
    ///
    /// When Unicode mode is enabled, case insensitivity is Unicode-aware.
    /// Specifically, it will apply the "simple" case folding rules as
    /// specified by Unicode.
    ///
    /// By default this is disabled. It may alternatively be selectively
    /// enabled in the regular expression itself via the `i` flag.
    pub fn case_insensitive(mut self, yes: bool) -> SyntaxConfig {
        self.case_insensitive = yes;
        self
    }

    /// Enable or disable the multi-line matching flag by default.
    ///
    /// When this is enabled, the `^` and `$` look-around assertions will
    /// match immediately after and immediately before a new line character,
    /// respectively. Note that the `\A` and `\z` look-around assertions are
    /// unaffected by this setting and always correspond to matching at the
    /// beginning and end of the input.
    ///
    /// By default this is disabled. It may alternatively be selectively
    /// enabled in the regular expression itself via the `m` flag.
    pub fn multi_line(mut self, yes: bool) -> SyntaxConfig {
        self.multi_line = yes;
        self
    }

    /// Enable or disable the "dot matches any character" flag by default.
    ///
    /// When this is enabled, `.` will match any character. When it's disabled,
    /// then `.` will match any character except for a new line character.
    ///
    /// Note that `.` is impacted by whether the "unicode" setting is enabled
    /// or not. When Unicode is enabled (the defualt), `.` will match any UTF-8
    /// encoding of any Unicode scalar value (sans a new line, depending on
    /// whether this "dot matches new line" option is enabled). When Unicode
    /// mode is disabled, `.` will match any byte instead. Because of this,
    /// when Unicode mode is disabled, `.` can only be used when the "allow
    /// invalid UTF-8" option is enabled, since `.` could otherwise match
    /// invalid UTF-8.
    ///
    /// By default this is disabled. It may alternatively be selectively
    /// enabled in the regular expression itself via the `s` flag.
    pub fn dot_matches_new_line(mut self, yes: bool) -> SyntaxConfig {
        self.dot_matches_new_line = yes;
        self
    }

    /// Enable or disable the "swap greed" flag by default.
    ///
    /// When this is enabled, `.*` (for example) will become ungreedy and `.*?`
    /// will become greedy.
    ///
    /// By default this is disabled. It may alternatively be selectively
    /// enabled in the regular expression itself via the `U` flag.
    pub fn swap_greed(mut self, yes: bool) -> SyntaxConfig {
        self.swap_greed = yes;
        self
    }

    /// Enable verbose mode in the regular expression.
    ///
    /// When enabled, verbose mode permits insigificant whitespace in many
    /// places in the regular expression, as well as comments. Comments are
    /// started using `#` and continue until the end of the line.
    ///
    /// By default, this is disabled. It may be selectively enabled in the
    /// regular expression by using the `x` flag regardless of this setting.
    pub fn ignore_whitespace(mut self, yes: bool) -> SyntaxConfig {
        self.ignore_whitespace = yes;
        self
    }

    /// Enable or disable the Unicode flag (`u`) by default.
    ///
    /// By default this is **enabled**. It may alternatively be selectively
    /// disabled in the regular expression itself via the `u` flag.
    ///
    /// Note that unless "allow invalid UTF-8" is enabled (it's disabled by
    /// default), a regular expression will fail to parse if Unicode mode is
    /// disabled and a sub-expression could possibly match invalid UTF-8.
    ///
    /// **WARNING**: Unicode mode can greatly increase the size of the compiled
    /// DFA, which can noticeably impact both memory usage and compilation
    /// time. This is especially noticeable if your regex contains character
    /// classes like `\w` that are impacted by whether Unicode is enabled or
    /// not. If Unicode is not necessary, you are encouraged to disable it.
    pub fn unicode(mut self, yes: bool) -> SyntaxConfig {
        self.unicode = yes;
        self
    }

    /// When disabled, the builder will permit the construction of a regular
    /// expression that may match invalid UTF-8.
    ///
    /// For example, when [`SyntaxConfig::unicode`] is disabled, then
    /// expressions like `[^a]` may match invalid UTF-8 since they can match
    /// any single byte that is not `a`. By default, these sub-expressions
    /// are disallowed to avoid returning offsets that split a UTF-8
    /// encoded codepoint. However, in cases where matching at arbitrary
    /// locations is desired, this option can be disabled to permit all such
    /// sub-expressions.
    ///
    /// When enabled (the default), the builder is guaranteed to produce a
    /// regex that will only ever match valid UTF-8 (otherwise, the builder
    /// will return an error).
    pub fn utf8(mut self, yes: bool) -> SyntaxConfig {
        self.utf8 = yes;
        self
    }

    /// Set the nesting limit used for the regular expression parser.
    ///
    /// The nesting limit controls how deep the abstract syntax tree is allowed
    /// to be. If the AST exceeds the given limit (e.g., with too many nested
    /// groups), then an error is returned by the parser.
    ///
    /// The purpose of this limit is to act as a heuristic to prevent stack
    /// overflow when building a finite automaton from a regular expression's
    /// abstract syntax tree. In particular, construction currently uses
    /// recursion. In the future, the implementation may stop using recursion
    /// and this option will no longer be necessary.
    ///
    /// This limit is not checked until the entire AST is parsed. Therefore,
    /// if callers want to put a limit on the amount of heap space used, then
    /// they should impose a limit on the length, in bytes, of the concrete
    /// pattern string. In particular, this is viable since the parser will
    /// limit itself to heap space proportional to the lenth of the pattern
    /// string.
    ///
    /// Note that a nest limit of `0` will return a nest limit error for most
    /// patterns but not all. For example, a nest limit of `0` permits `a` but
    /// not `ab`, since `ab` requires a concatenation AST item, which results
    /// in a nest depth of `1`. In general, a nest limit is not something that
    /// manifests in an obvious way in the concrete syntax, therefore, it
    /// should not be used in a granular way.
    pub fn nest_limit(mut self, limit: u32) -> SyntaxConfig {
        self.nest_limit = limit;
        self
    }

    /// Whether to support octal syntax or not.
    ///
    /// Octal syntax is a little-known way of uttering Unicode codepoints in
    /// a regular expression. For example, `a`, `\x61`, `\u0061` and
    /// `\141` are all equivalent regular expressions, where the last example
    /// shows octal syntax.
    ///
    /// While supporting octal syntax isn't in and of itself a problem, it does
    /// make good error messages harder. That is, in PCRE based regex engines,
    /// syntax like `\1` invokes a backreference, which is explicitly
    /// unsupported in Rust's regex engine. However, many users expect it to
    /// be supported. Therefore, when octal support is disabled, the error
    /// message will explicitly mention that backreferences aren't supported.
    ///
    /// Octal syntax is disabled by default.
    pub fn octal(mut self, yes: bool) -> SyntaxConfig {
        self.octal = yes;
        self
    }

    /// Returns whether "unicode" mode is enabled.
    pub fn get_unicode(&self) -> bool {
        self.unicode
    }

    /// Returns whether "case insensitive" mode is enabled.
    pub fn get_case_insensitive(&self) -> bool {
        self.case_insensitive
    }

    /// Returns whether "multi line" mode is enabled.
    pub fn get_multi_line(&self) -> bool {
        self.multi_line
    }

    /// Returns whether "dot matches new line" mode is enabled.
    pub fn get_dot_matches_new_line(&self) -> bool {
        self.dot_matches_new_line
    }

    /// Returns whether "swap greed" mode is enabled.
    pub fn get_swap_greed(&self) -> bool {
        self.swap_greed
    }

    /// Returns whether "ignore whitespace" mode is enabled.
    pub fn get_ignore_whitespace(&self) -> bool {
        self.ignore_whitespace
    }

    /// Returns whether UTF-8 mode is enabled.
    pub fn get_utf8(&self) -> bool {
        self.utf8
    }

    /// Returns the "nest limit" setting.
    pub fn get_nest_limit(&self) -> u32 {
        self.nest_limit
    }

    /// Returns whether "octal" mode is enabled.
    pub fn get_octal(&self) -> bool {
        self.octal
    }

    /// Applies this configuration to the given parser.
    pub(crate) fn apply(&self, builder: &mut ParserBuilder) {
        builder
            .unicode(self.unicode)
            .case_insensitive(self.case_insensitive)
            .multi_line(self.multi_line)
            .dot_matches_new_line(self.dot_matches_new_line)
            .swap_greed(self.swap_greed)
            .ignore_whitespace(self.ignore_whitespace)
            .allow_invalid_utf8(!self.utf8)
            .nest_limit(self.nest_limit)
            .octal(self.octal);
    }
}

impl Default for SyntaxConfig {
    fn default() -> SyntaxConfig {
        SyntaxConfig::new()
    }
}
