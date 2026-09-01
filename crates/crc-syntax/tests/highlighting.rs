use crc_syntax::{HighlightSpan, Language, SyntaxTree, resolve, role_for};
use crc_theme::Highlight;

/// Parse and return the highlight of the first occurrence of `needle`.
fn role_of(language: Language, source: &str, needle: &str) -> Option<Highlight> {
    let mut tree = SyntaxTree::new(language).expect("grammar loads");
    tree.parse(source).expect("parses");

    let at = source.find(needle).expect("needle is in the source");
    tree.highlights(source)
        .into_iter()
        .find(|span| span.range.start <= at && at < span.range.end)
        .map(|span| span.highlight)
}

#[test]
fn every_grammar_loads_with_its_query() {
    for language in Language::ALL {
        SyntaxTree::new(*language)
            .unwrap_or_else(|e| panic!("{} failed to load: {e}", language.name()));
    }
}

#[test]
fn detects_the_language_from_the_file_name() {
    assert_eq!(Language::from_path("src/main.rs"), Some(Language::Rust));
    assert_eq!(
        Language::from_path("app/Editor.tsx"),
        Some(Language::Tsx),
        "tsx is its own grammar, not typescript with a flag"
    );
    assert_eq!(
        Language::from_path("core/useEditor.ts"),
        Some(Language::TypeScript)
    );
    assert_eq!(
        Language::from_path("vite.config.MJS"),
        Some(Language::JavaScript),
        "extensions are matched case-insensitively"
    );
    assert_eq!(Language::from_path("package.json"), Some(Language::Json));

    // No grammar is not an error — the buffer just renders as plain text.
    assert_eq!(Language::from_path("notes.txt"), None);
    assert_eq!(Language::from_path("Makefile"), None);
}

#[test]
fn highlights_rust() {
    let source = "fn main() {\n    let x = \"hi\"; // note\n}\n";

    assert_eq!(
        role_of(Language::Rust, source, "fn"),
        Some(Highlight::Keyword)
    );
    assert_eq!(
        role_of(Language::Rust, source, "main"),
        Some(Highlight::Function)
    );
    assert_eq!(
        role_of(Language::Rust, source, "\"hi\""),
        Some(Highlight::String)
    );
    assert_eq!(
        role_of(Language::Rust, source, "// note"),
        Some(Highlight::Comment)
    );
}

#[test]
fn highlights_typescript_including_what_javascript_provides() {
    // `interface` comes from the TypeScript query; `const` and the string come
    // from the JavaScript one. Both have to be in play.
    let source = "interface Props { path: string }\nconst greeting = 'hi'\n";

    assert_eq!(
        role_of(Language::TypeScript, source, "interface"),
        Some(Highlight::Keyword)
    );
    assert_eq!(
        role_of(Language::TypeScript, source, "const"),
        Some(Highlight::Keyword),
        "the javascript query has to be prepended, or half the file goes plain"
    );
    assert_eq!(
        role_of(Language::TypeScript, source, "'hi'"),
        Some(Highlight::String)
    );
}

#[test]
fn highlights_jsx_tags_in_tsx() {
    let source = "export function Editor() {\n  return <Surface density=\"balanced\" />\n}\n";

    assert_eq!(
        role_of(Language::Tsx, source, "Surface"),
        Some(Highlight::Function),
        "component tags read as the same blue the design gives function names"
    );
    assert_eq!(
        role_of(Language::Tsx, source, "density"),
        Some(Highlight::Parameter)
    );
}

#[test]
fn highlights_json() {
    let source = r#"{ "name": "crc-code", "version": 1 }"#;

    assert_eq!(
        role_of(Language::Json, source, "\"name\""),
        Some(Highlight::Parameter),
        "keys read as parameters, not as plain strings"
    );
    assert_eq!(
        role_of(Language::Json, source, "1"),
        Some(Highlight::Number)
    );
}

#[test]
fn spans_never_overlap_and_stay_in_order() {
    let source = "export function useEditor(path: string) {\n  return read(path) // ok\n}\n";
    let mut tree = SyntaxTree::new(Language::TypeScript).unwrap();
    tree.parse(source).unwrap();

    let spans = tree.highlights(source);
    assert!(!spans.is_empty());

    let mut previous_end = 0;
    for span in &spans {
        assert!(span.range.start < span.range.end, "empty span {span:?}");
        assert!(
            span.range.start >= previous_end,
            "{span:?} overlaps the span before it"
        );
        assert!(span.range.end <= source.len(), "{span:?} runs off the end");
        previous_end = span.range.end;
    }
}

#[test]
fn highlights_only_the_range_that_was_asked_for() {
    let source = "const a = 1\nconst b = 2\nconst c = 3\n";
    let mut tree = SyntaxTree::new(Language::JavaScript).unwrap();
    tree.parse(source).unwrap();

    let second_line = 12..23;
    let spans = tree.highlights_in(source, second_line.clone());

    assert!(!spans.is_empty(), "the middle line has tokens");
    for span in spans {
        assert!(
            span.range.start < second_line.end && span.range.end > second_line.start,
            "{span:?} is outside the requested range"
        );
    }
}

#[test]
fn reparses_incrementally_after_an_edit() {
    let before = "const greeting = 'hello'\n";
    let after = "const greeting = 'hello world'\n";

    let mut tree = SyntaxTree::new(Language::TypeScript).unwrap();
    tree.parse(before).unwrap();

    // Insert " world" before the closing quote.
    let start = before.find("'hello").unwrap() + "'hello".len();
    tree.edit(before, after, start, start, start + " world".len());
    tree.parse(after).unwrap();

    assert!(!tree.has_error());
    let at = after.find("'hello world'").unwrap();
    let span = tree
        .highlights(after)
        .into_iter()
        .find(|s| s.range.contains(&at))
        .expect("the string is still highlighted");
    assert_eq!(span.highlight, Highlight::String);
    assert_eq!(span.range, at..at + "'hello world'".len());
}

#[test]
fn a_half_typed_line_still_parses() {
    let mut tree = SyntaxTree::new(Language::Rust).unwrap();
    tree.parse("fn main() { let x = ").unwrap();

    assert!(
        tree.has_error(),
        "an unfinished line is an error to the parser"
    );
    assert!(
        !tree.highlights("fn main() { let x = ").is_empty(),
        "but it still colours what it understood"
    );
}

#[test]
fn nothing_is_highlighted_before_the_first_parse() {
    let tree = SyntaxTree::new(Language::Rust).unwrap();
    assert!(tree.highlights("fn main() {}").is_empty());
    assert!(!tree.has_error());
}

mod capture_names {
    use super::*;

    #[test]
    fn maps_the_names_grammars_actually_use() {
        assert_eq!(role_for("keyword.control.conditional"), Highlight::Keyword);
        assert_eq!(role_for("function.method"), Highlight::Function);
        assert_eq!(role_for("string.special.path"), Highlight::String);
        assert_eq!(role_for("comment.documentation"), Highlight::Comment);
        assert_eq!(role_for("punctuation.bracket"), Highlight::Punctuation);
        assert_eq!(role_for("constant.builtin"), Highlight::Number);
    }

    #[test]
    fn a_parameter_is_not_just_any_variable() {
        assert_eq!(role_for("variable.parameter"), Highlight::Parameter);
        assert_eq!(role_for("variable"), Highlight::Text);
        assert_eq!(role_for("variable.builtin"), Highlight::Text);
    }

    #[test]
    fn an_unknown_capture_falls_back_to_plain_text() {
        assert_eq!(role_for("something.a.grammar.invented"), Highlight::Text);
        assert_eq!(role_for(""), Highlight::Text);
    }
}

mod flattening {
    use super::*;

    fn spans(
        input: Vec<(std::ops::Range<usize>, Highlight)>,
    ) -> Vec<(std::ops::Range<usize>, Highlight)> {
        resolve(input)
            .into_iter()
            .map(|HighlightSpan { range, highlight }| (range, highlight))
            .collect()
    }

    #[test]
    fn an_inner_capture_wins_its_own_bytes_and_the_outer_keeps_the_rest() {
        let out = spans(vec![
            (0..10, Highlight::Function),
            (4..6, Highlight::Parameter),
        ]);

        assert_eq!(
            out,
            vec![
                (0..4, Highlight::Function),
                (4..6, Highlight::Parameter),
                (6..10, Highlight::Function),
            ]
        );
    }

    #[test]
    fn disjoint_captures_pass_through() {
        let out = spans(vec![(0..3, Highlight::Keyword), (5..9, Highlight::String)]);
        assert_eq!(
            out,
            vec![(0..3, Highlight::Keyword), (5..9, Highlight::String)]
        );
    }

    #[test]
    fn touching_runs_of_the_same_role_merge() {
        let out = spans(vec![(0..3, Highlight::Keyword), (3..6, Highlight::Keyword)]);
        assert_eq!(out, vec![(0..6, Highlight::Keyword)], "one span, not two");
    }

    #[test]
    fn order_of_arrival_does_not_matter() {
        let forwards = spans(vec![
            (0..10, Highlight::Function),
            (4..6, Highlight::Parameter),
        ]);
        let backwards = spans(vec![
            (4..6, Highlight::Parameter),
            (0..10, Highlight::Function),
        ]);
        assert_eq!(forwards, backwards);
    }

    #[test]
    fn nothing_in_nothing_out() {
        assert!(resolve(Vec::new()).is_empty());
    }
}
