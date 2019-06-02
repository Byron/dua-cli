mod app {
    use dua::interactive::App;
    use dua::{ByteFormat, Color, WalkOptions};
    use failure::Error;
    use std::path::Path;
    use tui::backend::TestBackend;
    use tui::Terminal;

    #[test]
    fn journey_with_single_path() -> Result<(), Error> {
        let mut terminal = Terminal::new(TestBackend::new(40, 20))?;
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample-01");

        let app = App::initialize(
            &mut terminal,
            WalkOptions {
                threads: 1,
                byte_format: ByteFormat::Metric,
                color: Color::None,
            },
            vec![input],
        )?;
        Ok(())
    }
}
