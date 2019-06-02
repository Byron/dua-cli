mod app {
    use dua::interactive::{EntryData, GraphIndexType, TerminalApp, Tree};
    use dua::{ByteFormat, Color, Sorting, WalkOptions};
    use failure::Error;
    use petgraph::prelude::NodeIndex;
    use pretty_assertions::assert_eq;
    use std::{ffi::OsString, fmt, path::Path};
    use tui::backend::TestBackend;
    use tui::Terminal;

    fn debug(item: impl fmt::Debug) -> String {
        format!("{:?}", item)
    }

    #[test]
    fn journey_with_single_path() -> Result<(), Error> {
        let (_, app) = initialized_app_and_terminal("sample-01")?;
        let expected_tree = sample_01_tree();

        assert_eq!(
            debug(app.tree.raw_edges()),
            debug(expected_tree.raw_edges()),
            "filesystem graph is stable and matches the directory structure"
        );
        Ok(())
    }

    fn initialized_app_and_terminal(
        fixture_path: &str,
    ) -> Result<(Terminal<TestBackend>, TerminalApp), Error> {
        let mut terminal = Terminal::new(TestBackend::new(40, 20))?;
        let input = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(fixture_path);
        let app = TerminalApp::initialize(
            &mut terminal,
            WalkOptions {
                threads: 1,
                byte_format: ByteFormat::Metric,
                color: Color::None,
                sorting: Sorting::AlphabeticalByFileName,
            },
            vec![input],
        )?;
        Ok((terminal, app))
    }

    fn sample_01_tree() -> Tree {
        let mut t = Tree::new();
        let mut add_node = |name, size, maybe_from_idx: Option<NodeIndex<GraphIndexType>>| {
            let n = t.add_node(EntryData {
                name: OsString::from(name),
                size,
                metadata_io_error: false,
            });
            if let Some(from) = maybe_from_idx {
                t.add_edge(from, n, ());
            }
            n
        };
        let r = add_node("", 0, None);
        {
            let s = add_node("sample-01", 0, Some(r));
            {
                add_node(".hidden.666", 666, Some(s));
                add_node("a", 256, Some(s));
                add_node("b.empty", 0, Some(s));
                add_node("c.lnk", 1, Some(s));
                let d = add_node("dir", 0, Some(s));
                {
                    add_node("1000bytes", 1000, Some(d));
                    add_node("dir-a.1mb", 1_000_000, Some(d));
                    add_node("dir-a.kb", 1024, Some(d));
                    let e = add_node("empty-dir", 0, Some(d));
                    {
                        add_node(".gitkeep", 0, Some(e));
                    }
                    let sub = add_node("sub", 0, Some(d));
                    {
                        add_node("dir-sub-a.256kb", 256_000, Some(sub));
                    }
                }
            }
        }
        t
    }
}
