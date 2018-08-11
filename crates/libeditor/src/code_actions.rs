use {TextRange, TextUnit, File, EditBuilder, Edit};
use libsyntax2::{
    ast::AstNode,
    SyntaxKind::COMMA,
    SyntaxNodeRef,
    algo::find_leaf_at_offset,
};
use itertools::unfold;

pub enum Action {
    Applicable,
    Applied(Edit),
}

fn flip_comma(file: &File, offset: TextUnit, apply: bool) -> Option<Action> {
    let syntax = file.syntax();
    let syntax = syntax.as_ref();

    let comma = find_leaf_at_offset(syntax, offset).find(|leaf| leaf.kind() == COMMA)?;
    let left = non_trivia_sibling(comma, Direction::Left)?;
    let right = non_trivia_sibling(comma, Direction::Right)?;
    if !apply {
        return Some(Action::Applicable);
    }

    let mut edit = EditBuilder::new();
    edit.replace(left.range(), right.text());
    edit.replace(right.range(), left.text());

    Some(Action::Applied(edit.finish()))
}

enum Direction {
    Left,
    Right,
}

fn non_trivia_sibling(node: SyntaxNodeRef, direction: Direction) -> Option<SyntaxNodeRef> {
    siblings(node, direction).find(|node| node.kind().is_trivia())
}

fn siblings(node: SyntaxNodeRef, direction: Direction) -> impl Iterator<Item=SyntaxNodeRef> {
    unfold(node.next_sibling(), |node| {
        node.take().map(|n| {
            *node = n.next_sibling();
            n
        })
    })
}

fn siblings2(node: SyntaxNodeRef, direction: Direction) -> impl Iterator<Item=SyntaxNodeRef> {
    generate(node.next_sibling(), |n| n.next_sibling())
}


fn generate<T>(first: Option<T>, step: impl Fn(&T) -> Option<T>) -> impl Iterator<Item=T> {
    unfold(first, move |slot| {
        slot.take().map(|curr| {
            *slot = step(&curr);
            curr
        })
    })
}
