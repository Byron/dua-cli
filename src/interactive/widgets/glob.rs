use anyhow::Result;
use crosstermion::input::Key;
use dua::traverse::{Tree, TreeIndex};
use globset::{Glob, GlobMatcher};
use petgraph::Direction;
use std::borrow::Borrow;
use std::path::PathBuf;
use tui::backend::Backend;
use tui::{
    layout::Rect,
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};
use tui_react::Terminal;

use crate::interactive::Cursor;

pub struct GlobPaneProps {
    pub border_style: Style,
    pub has_focus: bool,
}

#[derive(Default)]
pub struct GlobPane {
    pub input: String,
    cursor_position: usize,
}

impl GlobPane {
    pub fn process_events(&mut self, key: Key) {
        use crosstermion::input::Key::*;

        match key {
            Char(to_insert) => {
                self.enter_char(to_insert);
            }
            Backspace => {
                self.delete_char();
            }
            Left => {
                self.move_cursor_left();
            }
            Right => {
                self.move_cursor_right();
            }
            _ => {}
        };
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);

        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    pub fn render<B>(
        &mut self,
        props: impl Borrow<GlobPaneProps>,
        area: Rect,
        terminal: &mut Terminal<B>,
        cursor: &mut Cursor,
    ) where
        B: Backend,
    {
        let GlobPaneProps {
            border_style,
            has_focus,
        } = props.borrow();

        let title = "Glob search";
        let block = Block::default()
            .title(title)
            .border_style(*border_style)
            .borders(Borders::ALL);
        let inner_block_area = block.inner(area);
        block.render(area, terminal.current_buffer_mut());

        let spans = vec![Span::from(&self.input)];
        Paragraph::new(Text::from(Line::from(spans)))
            .style(Style::default())
            .render(
                margin_left_right(inner_block_area, 1),
                terminal.current_buffer_mut(),
            );

        if *has_focus {
            cursor.show = true;
            cursor.x = inner_block_area.x + self.cursor_position as u16 + 1;
            cursor.y = inner_block_area.y;
        } else {
            cursor.show = false;
        }
    }
}

fn margin_left_right(r: Rect, margin: u16) -> Rect {
    Rect {
        x: r.x + margin,
        y: r.y,
        width: r.width - 2 * margin,
        height: r.height,
    }
}

fn glob_search_neighbours(
    results: &mut Vec<TreeIndex>,
    tree: &Tree,
    root_index: TreeIndex,
    glob: &GlobMatcher,
    path: &mut PathBuf,
) {
    let iter = tree.neighbors_directed(root_index, Direction::Outgoing);
    for node_index in iter {
        if let Some(node) = tree.node_weight(node_index) {
            path.push(&node.name);
            // println!("{path:?}");
            if glob.is_match(&path) {
                // println!("match");
                results.push(node_index);
            } else {
                glob_search_neighbours(results, tree, node_index, glob, path);
            }
            path.pop();
        }
    }
}

pub fn glob_search(tree: &Tree, root_index: TreeIndex, glob: &str) -> Result<Vec<TreeIndex>> {
    let glob = Glob::new(glob)?.compile_matcher();
    let mut results = Vec::new();
    let mut path = PathBuf::new();
    glob_search_neighbours(&mut results, tree, root_index, &glob, &mut path);
    Ok(results)
}
