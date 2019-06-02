mod app {
    use dua::interactive::{EntryData, TerminalApp, Tree};
    use dua::{ByteFormat, Color, Sorting, WalkOptions};
    use failure::Error;
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
            debug(app.tree),
            debug(expected_tree),
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
        let mut expected_tree = Tree::new();
        expected_tree.add_node(EntryData {
            name: OsString::from("foo"),
            size: 231,
            metadata_io_error: false,
        });
        expected_tree
    }
}
