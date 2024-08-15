use std::path::{Path, PathBuf};
use std::time::Duration;

use durable_runtime::Config;

mod basic;

pub fn test_config() -> Config {
    Config::default()
        .suspend_margin(Duration::from_secs(1))
        .suspend_timeout(Duration::from_secs(1))
}

pub fn test_binary(name: impl AsRef<Path>) -> PathBuf {
    let Some(bindir) = std::env::var_os("DURABLE_TEST_BIN_DIR") else {
        panic!(
            "DURABLE_TEST_BIN_DIR env var is not set. Are you running tests without using `cargo \
             nextest run`?"
        );
    };

    let name = name.as_ref();
    let mut path = PathBuf::from(bindir);
    path.push(name);

    if !path.exists() {
        panic!(
            "Attempted to request non-existant test binary `{}`",
            name.display()
        );
    }

    path
}
