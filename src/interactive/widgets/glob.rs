use crate::interactive::widgets::tui_ext::{
    draw_text_nowrap_fn,
    util::{block_width, rect},
};
use anyhow::{Context, Result, anyhow};
use bstr::BString;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use dua::{
    KeysConfig,
    traverse::{Tree, TreeIndex},
};
use gix::glob::pattern::Case;
use petgraph::Direction;
use std::borrow::Borrow;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::interactive::state::Cursor;

pub struct GlobPaneProps<'a> {
    pub border_style: Style,
    pub has_focus: bool,
    pub keys: &'a KeysConfig,
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
            input: String::new(),
            cursor_grapheme_idx: 0,
            case: Case::Fold,
        }
    }
}

impl GlobPane {
    pub fn process_events(&mut self, key: KeyEvent, keys: &KeysConfig) {
        if key.kind == KeyEventKind::Release {
            return;
        }

        if keys.search_toggle_case.matches(key) {
            self.case = match self.case {
                Case::Sensitive => Case::Fold,
                Case::Fold => Case::Sensitive,
            };
        } else if keys.search_backspace.matches(key) {
            self.delete_char();
        } else if keys.search_left.matches(key) {
            self.move_cursor_left();
        } else if keys.search_right.matches(key) {
            self.move_cursor_right();
        } else if let KeyCode::Char(to_insert) = key.code {
            self.enter_char(to_insert);
        }
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
                .map(str::len)
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

    pub fn render<'a>(
        &mut self,
        props: impl Borrow<GlobPaneProps<'a>>,
        area: Rect,
        buffer: &mut Buffer,
        cursor: &mut Cursor,
    ) {
        let GlobPaneProps {
            border_style,
            has_focus,
            keys,
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
            draw_top_right_help(area, title, buffer, keys);

            cursor.show = true;
            cursor.x = inner_block_area.x
                + self
                    .input
                    .graphemes(true)
                    .take(self.cursor_grapheme_idx)
                    .map(UnicodeWidthStr::width)
                    .sum::<usize>() as u16
                + 1;
            cursor.y = inner_block_area.y;
        } else {
            cursor.show = false;
        }
    }
}

fn draw_top_right_help(area: Rect, title: &str, buf: &mut Buffer, keys: &KeysConfig) -> Rect {
    let help_text = format!(
        " search = {} | case = {} | cancel = {} ",
        keys.search_confirm, keys.search_toggle_case, keys.close_pane
    );
    let help_text_block_width = block_width(&help_text);
    let bound = Rect {
        width: area.width.saturating_sub(1),
        ..area
    };
    if block_width(title) + help_text_block_width <= bound.width {
        draw_text_nowrap_fn(
            rect::snap_to_right(bound, help_text_block_width),
            buf,
            &help_text,
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
    glob: &gix::glob::Pattern,
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
            path.extend_from_slice(gix::path::into_bstr(&node.name).as_ref());
            if glob.matches_repo_relative_path(
                path.as_ref(),
                basename_start,
                Some(node.is_dir),
                case,
                gix::glob::wildmatch::Mode::NO_MATCH_SLASH_LITERAL,
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
    case: gix::glob::pattern::Case,
) -> Result<Vec<TreeIndex>> {
    let glob = gix::glob::Pattern::from_bytes_without_negation(glob.as_bytes())
        .with_context(|| anyhow!("Glob was empty or only whitespace"))?;
    let mut results = Vec::new();
    let mut path = BString::default();
    glob_search_neighbours(&mut results, tree, root_index, &glob, &mut path, case);
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEventKind, KeyEventState, KeyModifiers};

    #[test]
    fn default_toggle_case_key_does_not_type_into_input() {
        let mut glob_pane = GlobPane::default();
        let keys = KeysConfig::default();
        assert_eq!(glob_pane.input, "");
        assert_eq!(glob_pane.case, Case::Fold); // default is case-insensitive

        let ctrl_f = KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        };
        glob_pane.process_events(ctrl_f, &keys);
        assert_eq!(glob_pane.case, Case::Sensitive);
        assert_eq!(glob_pane.input, "");

        glob_pane.process_events(ctrl_f, &keys);
        assert_eq!(glob_pane.case, Case::Fold);
    }

    #[test]
    fn configured_character_bindings_take_precedence_over_text_input() {
        let keys = toml::from_str::<dua::Config>(
            r#"
            [keys]
            search_toggle_case = ["t"]
            search_backspace = ["x"]
            search_left = ["h"]
            search_right = ["l"]
            "#,
        )
        .expect("valid config")
        .keys;
        let mut glob_pane = GlobPane::default();

        glob_pane.process_events(KeyCode::Char('a').into(), &keys);
        glob_pane.process_events(KeyCode::Char('h').into(), &keys);
        assert_eq!(glob_pane.cursor_grapheme_idx, 0);
        glob_pane.process_events(KeyCode::Char('l').into(), &keys);
        assert_eq!(glob_pane.cursor_grapheme_idx, 1);
        glob_pane.process_events(KeyCode::Char('x').into(), &keys);
        glob_pane.process_events(KeyCode::Char('t').into(), &keys);

        assert_eq!(
            glob_pane.input, "",
            "configured bindings should not be typed into the input"
        );
        assert_eq!(
            glob_pane.case,
            Case::Sensitive,
            "configured toggle binding should change case sensitivity"
        );
    }

    #[test]
    fn rendered_help_uses_configured_bindings() {
        let keys = toml::from_str::<dua::Config>(
            r#"
            [keys]
            close_pane = ["q"]
            search_confirm = ["f2"]
            search_toggle_case = ["alt+c"]
            "#,
        )
        .expect("valid config")
        .keys;
        let area = Rect::new(0, 0, 100, 3);
        let mut buffer = Buffer::empty(area);

        GlobPane::default().render(
            GlobPaneProps {
                border_style: Style::default(),
                has_focus: true,
                keys: &keys,
            },
            area,
            &mut buffer,
            &mut Cursor::default(),
        );

        insta::assert_debug_snapshot!(
            buffer,
            "glob pane help with configured bindings",
            @r#"
        Buffer {
            area: Rect { x: 0, y: 0, width: 100, height: 3 },
            content: [
                "┌Git-Glob (case-insensitive)────────────────────────── search = <F2> | case = Alt + c | cancel = q ┐",
                "│                                                                                                  │",
                "└──────────────────────────────────────────────────────────────────────────────────────────────────┘",
            ],
            styles: [
                x: 0, y: 0, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
            ]
        }
        "#
        );
    }
}
