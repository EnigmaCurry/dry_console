impl CommandLibrary {
    pub fn get_script(&self) -> &'static str {
        match self {
            CommandLibrary::TestExampleOne => include_str!("/var/home/ryan/git/vendor/EnigmaCurry/dry_console/script/src/scripts/test_example_one.sh"),
            _ => unreachable!("Unknown command library variant"),
        }
    }
}
