use anyhow::{Context, Result, anyhow};
use bstr::BString;
use crosstermion::crossterm::event::KeyEventKind;
use crosstermion::input::Key;
use dua::traverse::{Tree, TreeIndex};
use gix_glob::pattern::Case;
use petgraph::Direction;
use std::borrow::Borrow;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};
use tui_react::{
    draw_text_nowrap_fn,
    util::{block_width, rect},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::interactive::state::Cursor;

pub struct GlobPaneProps {
    pub border_style: Style,
    pub has_focus: bool,
}

pub struct GlobPane {
    pub input: String,
    /// The index of the grapheme the cursor currently points to.
    /// This hopefully rightfully assumes that a grapheme will be matching the block size on screen
    /// and is treated as 'one character'. If not, it will be off, which isn't the end of the world.
    // TODO: use `tui-textarea` for proper cursor handling, needs native crossterm events.
    cursor_grapheme_idx: usize,
    pub case: Case,
}

impl Default for GlobPane {
    fn default() -> Self {
        GlobPane {
            input: "".to_string(),
            cursor_grapheme_idx: 0,
            case: Case::Fold,
        }
    }
}

impl GlobPane {
    pub fn process_events(&mut self, key: Key) {
        use crosstermion::crossterm::event::KeyCode::*;
        use crosstermion::crossterm::event::KeyModifiers;
        if key.kind == KeyEventKind::Release {
            return;
        }
        match key.code {
            Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.case = match self.case {
                    Case::Sensitive => Case::Fold,
                    Case::Fold => Case::Sensitive,
                };
            }
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
        let cursor_moved_left = self.cursor_grapheme_idx.saturating_sub(1);
        self.cursor_grapheme_idx = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_grapheme_idx.saturating_add(1);
        self.cursor_grapheme_idx = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.input.insert(
            self.input
                .graphemes(true)
                .take(self.cursor_grapheme_idx)
                .map(|g| g.len())
                .sum::<usize>(),
            new_char,
        );

        for _ in 0..new_char.to_string().graphemes(true).count() {
            self.move_cursor_right();
        }
    }

    fn delete_char(&mut self) {
        if self.cursor_grapheme_idx == 0 {
            return;
        }

        let cur_idx = self.cursor_grapheme_idx;
        let before_char_to_delete = self.input.graphemes(true).take(cur_idx - 1);
        let after_char_to_delete = self.input.graphemes(true).skip(cur_idx);

        self.input = before_char_to_delete.chain(after_char_to_delete).collect();
        self.move_cursor_left();
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.graphemes(true).count())
    }

    pub fn render(
        &mut self,
        props: impl Borrow<GlobPaneProps>,
        area: Rect,
        buffer: &mut Buffer,
        cursor: &mut Cursor,
    ) {
        let GlobPaneProps {
            border_style,
            has_focus,
        } = props.borrow();

        let title = match self.case {
            Case::Sensitive => "Git-Glob (case-sensitive)",
            Case::Fold => "Git-Glob (case-insensitive)",
        };

        let block = Block::default()
            .title(title)
            .border_style(*border_style)
            .borders(Borders::ALL);
        let inner_block_area = block.inner(area);
        block.render(area, buffer);

        let spans = vec![Span::from(&self.input)];
        Paragraph::new(Text::from(Line::from(spans)))
            .style(Style::default())
            .render(margin_left_right(inner_block_area, 1), buffer);

        if *has_focus {
            draw_top_right_help(area, title, buffer);

            cursor.show = true;
            cursor.x = inner_block_area.x
                + self
                    .input
                    .graphemes(true)
                    .take(self.cursor_grapheme_idx)
                    .map(|g| g.width())
                    .sum::<usize>() as u16
                + 1;
            cursor.y = inner_block_area.y;
        } else {
            cursor.show = false;
        }
    }
}

fn draw_top_right_help(area: Rect, title: &str, buf: &mut Buffer) -> Rect {
    let help_text = " search = enter | case = ^f | cancel = esc ";
    let help_text_block_width = block_width(help_text);
    let bound = Rect {
        width: area.width.saturating_sub(1),
        ..area
    };
    if block_width(title) + help_text_block_width <= bound.width {
        draw_text_nowrap_fn(
            rect::snap_to_right(bound, help_text_block_width),
            buf,
            help_text,
            |_, _, _| Style::default(),
        );
    }
    bound
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
    glob: &gix_glob::Pattern,
    path: &mut BString,
    case: Case,
) {
    for node_index in tree.neighbors_directed(root_index, Direction::Outgoing) {
        if let Some(node) = tree.node_weight(node_index) {
            let previous_len = path.len();
            let basename_start = if path.is_empty() {
                None
            } else {
                path.push(b'/');
                Some(previous_len + 1)
            };
            path.extend_from_slice(gix_path::into_bstr(&node.name).as_ref());
            if glob.matches_repo_relative_path(
                path.as_ref(),
                basename_start,
                Some(node.is_dir),
                case,
                gix_glob::wildmatch::Mode::NO_MATCH_SLASH_LITERAL,
            ) {
                results.push(node_index);
            } else {
                glob_search_neighbours(results, tree, node_index, glob, path, case);
            }
            path.truncate(previous_len);
        }
    }
}

pub fn glob_search(
    tree: &Tree,
    root_index: TreeIndex,
    glob: &str,
    case: gix_glob::pattern::Case,
) -> Result<Vec<TreeIndex>> {
    let glob = gix_glob::Pattern::from_bytes_without_negation(glob.as_bytes())
        .with_context(|| anyhow!("Glob was empty or only whitespace"))?;
    let mut results = Vec::new();
    let mut path = Default::default();
    glob_search_neighbours(&mut results, tree, root_index, &glob, &mut path, case);
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crosstermion::crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};

    #[test]
    fn ctrl_f_key_types_into_input() {
        let mut glob_pane = GlobPane::default();
        assert_eq!(glob_pane.input, "");
        assert_eq!(glob_pane.case, Case::Fold); // default is case-insensitive

        let ctrl_f = Key {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: crosstermion::crossterm::event::KeyEventState::empty(),
        };
        glob_pane.process_events(ctrl_f);
        assert_eq!(glob_pane.case, Case::Sensitive);

        glob_pane.process_events(ctrl_f);
        assert_eq!(glob_pane.case, Case::Fold);
    }
}
