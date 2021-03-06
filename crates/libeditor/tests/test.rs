extern crate libeditor;
extern crate libsyntax2;
extern crate itertools;
#[macro_use]
extern crate assert_eq_text;

use std::fmt;
use itertools::Itertools;
use libeditor::{
    File, TextUnit, TextRange, ActionResult, CursorPosition,
    highlight, runnables, extend_selection, file_structure,
    flip_comma, add_derive, matching_brace,
};

#[test]
fn test_extend_selection() {
    let file = file(r#"fn foo() {
    1 + 1
}
"#);
    let range = TextRange::offset_len(18.into(), 0.into());
    let range = extend_selection(&file, range).unwrap();
    assert_eq!(range, TextRange::from_to(17.into(), 18.into()));
    let range = extend_selection(&file, range).unwrap();
    assert_eq!(range, TextRange::from_to(15.into(), 20.into()));
}

#[test]
fn test_highlighting() {
    let file = file(r#"
// comment
fn main() {}
    println!("Hello, {}!", 92);
"#);
    let hls = highlight(&file);
    dbg_eq(
        r#"[HighlightedRange { range: [1; 11), tag: "comment" },
            HighlightedRange { range: [12; 14), tag: "keyword" },
            HighlightedRange { range: [15; 19), tag: "function" },
            HighlightedRange { range: [29; 36), tag: "text" },
            HighlightedRange { range: [38; 50), tag: "string" },
            HighlightedRange { range: [52; 54), tag: "literal" }]"#,
        &hls,
    );
}

#[test]
fn test_runnables() {
    let file = file(r#"
fn main() {}

#[test]
fn test_foo() {}

#[test]
#[ignore]
fn test_foo() {}
"#);
    let runnables = runnables(&file);
    dbg_eq(
        r#"[Runnable { range: [1; 13), kind: Bin },
            Runnable { range: [15; 39), kind: Test { name: "test_foo" } },
            Runnable { range: [41; 75), kind: Test { name: "test_foo" } }]"#,
        &runnables,
    )
}

#[test]
fn test_file_structure() {
    let file = file(r#"
struct Foo {
    x: i32
}

mod m {
    fn bar() {}
}

enum E { X, Y(i32) }
type T = ();
static S: i32 = 92;
const C: i32 = 92;

impl E {}

impl fmt::Debug for E {}
"#);
    let symbols = file_structure(&file);
    dbg_eq(
        r#"[StructureNode { parent: None, label: "Foo", navigation_range: [8; 11), node_range: [1; 26), kind: STRUCT_DEF },
            StructureNode { parent: Some(0), label: "x", navigation_range: [18; 19), node_range: [18; 24), kind: NAMED_FIELD },
            StructureNode { parent: None, label: "m", navigation_range: [32; 33), node_range: [28; 53), kind: MODULE },
            StructureNode { parent: Some(2), label: "bar", navigation_range: [43; 46), node_range: [40; 51), kind: FN_DEF },
            StructureNode { parent: None, label: "E", navigation_range: [60; 61), node_range: [55; 75), kind: ENUM_DEF },
            StructureNode { parent: None, label: "T", navigation_range: [81; 82), node_range: [76; 88), kind: TYPE_DEF },
            StructureNode { parent: None, label: "S", navigation_range: [96; 97), node_range: [89; 108), kind: STATIC_DEF },
            StructureNode { parent: None, label: "C", navigation_range: [115; 116), node_range: [109; 127), kind: CONST_DEF },
            StructureNode { parent: None, label: "impl E", navigation_range: [134; 135), node_range: [129; 138), kind: IMPL_ITEM },
            StructureNode { parent: None, label: "impl fmt::Debug for E", navigation_range: [160; 161), node_range: [140; 164), kind: IMPL_ITEM }]"#,
        &symbols,
    )
}

#[test]
fn test_swap_comma() {
    check_action(
        "fn foo(x: i32,<|> y: Result<(), ()>) {}",
        "fn foo(y: Result<(), ()>,<|> x: i32) {}",
        |file, off| flip_comma(file, off).map(|f| f()),
    )
}

#[test]
fn test_add_derive() {
    check_action(
        "struct Foo { a: i32, <|>}",
        "#[derive(<|>)]\nstruct Foo { a: i32, }",
        |file, off| add_derive(file, off).map(|f| f()),
    );
    check_action(
        "#[derive(Clone)]\nstruct Foo { a: i32<|>, }",
        "#[derive(Clone<|>)]\nstruct Foo { a: i32, }",
        |file, off| add_derive(file, off).map(|f| f()),
    );
}

#[test]
fn test_matching_brace() {
    fn do_check(before: &str, after: &str) {
        let (pos, before) = extract_cursor(before);
        let file = file(&before);
        let new_pos = match matching_brace(&file, pos) {
            None => pos,
            Some(pos) => pos,
        };
        let actual = add_cursor(&before, new_pos);
        assert_eq_text!(after, &actual);
    }

    do_check(
        "struct Foo { a: i32, }<|>",
        "struct Foo <|>{ a: i32, }",
    );
}

fn file(text: &str) -> File {
    File::parse(text)
}

fn dbg_eq(expected: &str, actual: &impl fmt::Debug) {
    let actual = format!("{:?}", actual);
    let expected = expected.lines().map(|l| l.trim()).join(" ");
    assert_eq!(expected, actual);
}

fn check_action<F: Fn(&File, TextUnit) -> Option<ActionResult>>(
    before: &str,
    after: &str,
    f: F,
) {
    let (before_cursor_pos, before) = extract_cursor(before);
    let file = file(&before);
    let result = f(&file, before_cursor_pos).expect("code action is not applicable");
    let actual = result.edit.apply(&before);
    let actual_cursor_pos = match result.cursor_position {
        CursorPosition::Same => result.edit.apply_to_offset(before_cursor_pos).unwrap(),
        CursorPosition::Offset(off) => off,
    };
    let actual = add_cursor(&actual, actual_cursor_pos);
    assert_eq_text!(after, &actual);
}

fn extract_cursor(text: &str) -> (TextUnit, String) {
    let cursor = "<|>";
    let cursor_pos = match text.find(cursor) {
        None => panic!("text should contain cursor marker"),
        Some(pos) => pos,
    };
    let mut new_text = String::with_capacity(text.len() - cursor.len());
    new_text.push_str(&text[..cursor_pos]);
    new_text.push_str(&text[cursor_pos + cursor.len()..]);
    let cursor_pos = TextUnit::from(cursor_pos as u32);
    (cursor_pos, new_text)
}

fn add_cursor(text: &str, offset: TextUnit) -> String {
    let offset: u32 = offset.into();
    let offset: usize = offset as usize;
    let mut res = String::new();
    res.push_str(&text[..offset]);
    res.push_str("<|>");
    res.push_str(&text[offset..]);
    res
}
