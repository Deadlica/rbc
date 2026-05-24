use std::fs;
use std::path::PathBuf;

/// Fetch box art for a game, checking local cache first.
/// Returns the path to the cached image file, or None on failure.
pub fn get_art(game_name: &str) -> Option<PathBuf> {
    let cache_dir = dirs::config_dir()?.join("rbc").join("art");
    let safe_name = sanitize_filename(game_name);
    let cached = cache_dir.join(format!("{safe_name}.png"));

    if cached.exists() {
        return Some(cached);
    }

    // Try fetching from libretro thumbnails
    if let Some(data) = fetch_libretro(game_name) {
        fs::create_dir_all(&cache_dir).ok();
        fs::write(&cached, &data).ok();
        return Some(cached);
    }

    None
}

/// Save a custom art file (from URL or local path) to the cache.
pub fn set_custom_art(game_name: &str, source: &str) -> Option<PathBuf> {
    let cache_dir = dirs::config_dir()?.join("rbc").join("art");
    fs::create_dir_all(&cache_dir).ok();
    let safe_name = sanitize_filename(game_name);
    let cached = cache_dir.join(format!("{safe_name}.png"));

    if source.starts_with("http://") || source.starts_with("https://") {
        let data = fetch_url(source)?;
        fs::write(&cached, &data).ok()?;
    } else {
        // Local file — copy it
        fs::copy(source, &cached).ok()?;
    }

    Some(cached)
}

/// Fetch from libretro thumbnail repo.
fn fetch_libretro(game_name: &str) -> Option<Vec<u8>> {
    // libretro uses URL-encoded names with specific formatting
    let encoded = game_name.replace('&', "_");
    let url = format!(
        "https://thumbnails.libretro.com/Nintendo%20-%20Game%20Boy%20Color/Named_Boxarts/{}.png",
        urlencoded(&encoded)
    );

    fetch_url(&url).or_else(|| {
        // Try Game Boy (non-color) path
        let url = format!(
            "https://thumbnails.libretro.com/Nintendo%20-%20Game%20Boy/Named_Boxarts/{}.png",
            urlencoded(&encoded)
        );
        fetch_url(&url)
    })
}

/// Fetch a URL and return the body bytes.
fn fetch_url(url: &str) -> Option<Vec<u8>> {
    let resp = ureq::get(url).call().ok()?;
    if resp.status() != 200 { return None; }
    let mut body = Vec::new();
    resp.into_reader().read_to_end(&mut body).ok()?;
    Some(body)
}

/// URL-encode a string (minimal: spaces, parens, commas).
fn urlencoded(s: &str) -> String {
    s.replace(' ', "%20")
        .replace('(', "%28")
        .replace(')', "%29")
        .replace(',', "%2C")
        .replace('\'', "%27")
}

/// Make a filename safe by removing problematic characters.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}
