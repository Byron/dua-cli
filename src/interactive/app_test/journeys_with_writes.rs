use crate::interactive::app_test::utils::{
    initialized_app_and_terminal_from_paths, WritableFixture,
};
use failure::Error;

#[test]
fn basic_user_journey_with_deletion() -> Result<(), Error> {
    let fixture = WritableFixture::from("sample-02");
    let (mut _terminal, mut _app) =
        initialized_app_and_terminal_from_paths(&[fixture.root.clone()])?;
    Ok(())
}
