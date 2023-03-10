use crate::{EarlyContext, EarlyLintPass, LintContext};
use ast::util::unicode::{contains_text_flow_control_chars, TEXT_FLOW_CONTROL_CHARS};
use rustc_ast as ast;
use rustc_errors::{fluent, Applicability, SuggestionStyle};
use rustc_span::{BytePos, Span, Symbol};

declare_lint! {
    /// The `text_direction_codepoint_in_literal` lint detects Unicode codepoints that change the
    /// visual representation of text on screen in a way that does not correspond to their on
    /// memory representation.
    ///
    /// ### Explanation
    ///
    /// The unicode characters `\u{202A}`, `\u{202B}`, `\u{202D}`, `\u{202E}`, `\u{2066}`,
    /// `\u{2067}`, `\u{2068}`, `\u{202C}` and `\u{2069}` make the flow of text on screen change
    /// its direction on software that supports these codepoints. This makes the text "abc" display
    /// as "cba" on screen. By leveraging software that supports these, people can write specially
    /// crafted literals that make the surrounding code seem like it's performing one action, when
    /// in reality it is performing another. Because of this, we proactively lint against their
    /// presence to avoid surprises.
    ///
    /// ### Example
    ///
    /// ```rust,compile_fail
    /// #![deny(text_direction_codepoint_in_literal)]
    /// fn main() {
    ///     println!("{:?}", '‮');
    /// }
    /// ```
    ///
    /// {{produces}}
    ///
    pub TEXT_DIRECTION_CODEPOINT_IN_LITERAL,
    Deny,
    "detect special Unicode codepoints that affect the visual representation of text on screen, \
     changing the direction in which text flows",
}

declare_lint_pass!(HiddenUnicodeCodepoints => [TEXT_DIRECTION_CODEPOINT_IN_LITERAL]);

impl HiddenUnicodeCodepoints {
    fn lint_text_direction_codepoint(
        &self,
        cx: &EarlyContext<'_>,
        text: Symbol,
        span: Span,
        padding: u32,
        point_at_inner_spans: bool,
        label: &str,
    ) {
        // Obtain the `Span`s for each of the forbidden chars.
        let spans: Vec<_> = text
            .as_str()
            .char_indices()
            .filter_map(|(i, c)| {
                TEXT_FLOW_CONTROL_CHARS.contains(&c).then(|| {
                    let lo = span.lo() + BytePos(i as u32 + padding);
                    (c, span.with_lo(lo).with_hi(lo + BytePos(c.len_utf8() as u32)))
                })
            })
            .collect();

        cx.struct_span_lint(
            TEXT_DIRECTION_CODEPOINT_IN_LITERAL,
            span,
            fluent::lint_hidden_unicode_codepoints,
            |lint| {
                lint.set_arg("label", label);
                lint.set_arg("count", spans.len());
                lint.span_label(span, fluent::label);
                lint.note(fluent::note);
                if point_at_inner_spans {
                    for (c, span) in &spans {
                        lint.span_label(*span, format!("{:?}", c));
                    }
                }
                if point_at_inner_spans && !spans.is_empty() {
                    lint.multipart_suggestion_with_style(
                        fluent::suggestion_remove,
                        spans.iter().map(|(_, span)| (*span, "".to_string())).collect(),
                        Applicability::MachineApplicable,
                        SuggestionStyle::HideCodeAlways,
                    );
                    lint.multipart_suggestion(
                        fluent::suggestion_escape,
                        spans
                            .into_iter()
                            .map(|(c, span)| {
                                let c = format!("{:?}", c);
                                (span, c[1..c.len() - 1].to_string())
                            })
                            .collect(),
                        Applicability::MachineApplicable,
                    );
                } else {
                    // FIXME: in other suggestions we've reversed the inner spans of doc comments. We
                    // should do the same here to provide the same good suggestions as we do for
                    // literals above.
                    lint.set_arg(
                        "escaped",
                        spans
                            .into_iter()
                            .map(|(c, _)| format!("{:?}", c))
                            .collect::<Vec<String>>()
                            .join(", "),
                    );
                    lint.note(fluent::suggestion_remove);
                    lint.note(fluent::no_suggestion_note_escape);
                }
                lint
            },
        );
    }
}
impl EarlyLintPass for HiddenUnicodeCodepoints {
    fn check_attribute(&mut self, cx: &EarlyContext<'_>, attr: &ast::Attribute) {
        if let ast::AttrKind::DocComment(_, comment) = attr.kind {
            if contains_text_flow_control_chars(comment.as_str()) {
                self.lint_text_direction_codepoint(cx, comment, attr.span, 0, false, "doc comment");
            }
        }
    }

    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &ast::Expr) {
        // byte strings are already handled well enough by `EscapeError::NonAsciiCharInByteString`
        match &expr.kind {
            ast::ExprKind::Lit(token_lit) => {
                let text = token_lit.symbol;
                if !contains_text_flow_control_chars(text.as_str()) {
                    return;
                }
                let padding = match token_lit.kind {
                    // account for `"` or `'`
                    ast::token::LitKind::Str | ast::token::LitKind::Char => 1,
                    // account for `r###"`
                    ast::token::LitKind::StrRaw(n) => n as u32 + 2,
                    _ => return,
                };
                self.lint_text_direction_codepoint(cx, text, expr.span, padding, true, "literal");
            }
            _ => {}
        };
    }
}
