fn main() {
    let development = std::env::var_os("DURABLE_DEVELOPMENT")
        .map(|var| var == "1" || var == "true")
        .unwrap_or(false);

    if !development {
        println!("cargo::rustc-env=SQLX_OFFLINE=true");
        println!("cargo::rustc-env=SQLX_OFFLINE_DIR=.sqlx");
    }
}
