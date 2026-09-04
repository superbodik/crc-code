use crc_syntax::{Language, SyntaxTree};

fn parse(language: Language, text: &str) -> SyntaxTree {
    let mut tree = SyntaxTree::new(language).expect("a grammar");
    tree.parse(text).expect("a parse");
    tree
}

#[test]
fn a_sound_file_reports_nothing() {
    let tree = parse(Language::Rust, "fn main() {\n    let x = 1;\n}\n");

    assert!(!tree.has_error());
    assert!(tree.faults().is_empty());
}

#[test]
fn an_unclosed_block_is_reported() {
    let tree = parse(Language::Rust, "fn main() {\n    let x = 1;\n");

    assert!(
        !tree.faults().is_empty(),
        "an unclosed block must be reported"
    );
}

#[test]
fn a_token_the_grammar_expected_is_named_as_missing() {
    let tree = parse(Language::Rust, "fn main() { let x = 1 }");
    let missing = tree
        .faults()
        .into_iter()
        .find(|fault| fault.missing)
        .expect("the semicolon is missing, not merely wrong");

    assert_eq!(missing.kind, ";");
    assert_eq!(missing.message(), "не хватает ;");
}

#[test]
fn rubbish_the_grammar_cannot_place_is_not_called_missing() {
    let tree = parse(Language::Rust, "fn main() { let x = ; }");
    let faults = tree.faults();

    assert!(!faults.is_empty());
    assert!(
        faults.iter().all(|fault| !fault.missing),
        "nothing is missing here, the value is simply wrong"
    );
    assert_eq!(faults[0].message(), "разбор здесь ломается");
}

#[test]
fn rubbish_in_the_middle_points_at_the_line_it_is_on() {
    let tree = parse(Language::Rust, "fn one() {}\nfn ??? {}\nfn three() {}\n");
    let faults = tree.faults();

    assert!(!faults.is_empty());
    assert!(
        faults.iter().any(|fault| fault.line == 1),
        "the fault should sit on the second line, got {:?}",
        faults.iter().map(|f| f.line).collect::<Vec<_>>()
    );
}

#[test]
fn every_fault_carries_a_message_a_reader_can_use() {
    let tree = parse(Language::Rust, "fn main( {\n");

    for fault in tree.faults() {
        assert!(!fault.message().is_empty());
        assert!(fault.range.start <= fault.range.end);
    }
}

#[test]
fn other_languages_report_faults_too() {
    let python = parse(Language::Python, "def one(:\n    pass\n");
    assert!(!python.faults().is_empty(), "python");

    let json = parse(Language::Json, "{ \"a\": }");
    assert!(!json.faults().is_empty(), "json");
}

#[test]
fn a_flood_of_faults_is_capped() {
    let rubbish = "} ".repeat(4000);
    let tree = parse(Language::Rust, &rubbish);

    assert!(tree.faults().len() <= 200, "the list must stay readable");
}

#[test]
fn faults_are_reported_in_the_order_they_appear() {
    let tree = parse(Language::Rust, "fn a( {}\nfn b() {}\nfn c( {}\n");
    let faults = tree.faults();

    let lines: Vec<usize> = faults.iter().map(|fault| fault.line).collect();
    let mut sorted = lines.clone();
    sorted.sort_unstable();

    assert_eq!(lines, sorted, "faults came out of order: {lines:?}");
}
