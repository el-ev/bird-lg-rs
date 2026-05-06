use web_sys::UrlSearchParams;

pub const LOCALE_STORAGE_KEY: &str = "bird-lg-rs.autopeer.locale";

pub fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

pub fn hash_param(name: &str) -> Option<String> {
    web_sys::window().and_then(|window| {
        let hash = window.location().hash().ok()?;
        let query = hash.strip_prefix('#').unwrap_or(&hash);
        let params = UrlSearchParams::new_with_str(query).ok()?;
        params.get(name)
    })
}
