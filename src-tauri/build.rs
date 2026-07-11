use std::{collections::HashMap, env, fs, path::PathBuf};

fn main() {
    for name in [
        "PIXELDONE_SUPABASE_URL",
        "PIXELDONE_SUPABASE_PUBLISHABLE_KEY",
        "PIXELDONE_REQUIRE_CLOUD_CONFIG",
        "PIXELDONE_ALLOW_INSECURE_HTTP",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let android_properties = manifest_dir.join("../../PixelDone/local.properties");
    println!("cargo:rerun-if-changed={}", android_properties.display());
    let properties = read_properties(&android_properties);

    let supabase_url = config_value(
        "PIXELDONE_SUPABASE_URL",
        "pixeldone.supabaseUrl",
        &properties,
    );
    let publishable_key = config_value(
        "PIXELDONE_SUPABASE_PUBLISHABLE_KEY",
        "pixeldone.supabasePublishableKey",
        &properties,
    );
    let require_cloud = env_bool("PIXELDONE_REQUIRE_CLOUD_CONFIG", true);
    let allow_http = env_bool("PIXELDONE_ALLOW_INSECURE_HTTP", true);

    if require_cloud && (supabase_url.is_empty() || publishable_key.is_empty()) {
        panic!(
            "PixelDone formal builds require PIXELDONE_SUPABASE_URL and \
             PIXELDONE_SUPABASE_PUBLISHABLE_KEY (or the matching Android local.properties keys)."
        );
    }
    if !supabase_url.is_empty() && !supabase_url.starts_with("http://") {
        panic!("PixelDone 3.1 is configured for the approved direct-IP HTTP Supabase deployment.");
    }
    if !allow_http {
        panic!(
            "PIXELDONE_ALLOW_INSECURE_HTTP must remain true for the approved PixelDone deployment."
        );
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("build output dir"));
    let generated = format!(
        "pub const SUPABASE_URL: &str = {supabase_url:?};\n\
         pub const SUPABASE_PUBLISHABLE_KEY: &str = {publishable_key:?};\n\
         pub const ALLOW_INSECURE_HTTP: bool = true;\n"
    );
    fs::write(out_dir.join("pixeldone_cloud_config.rs"), generated)
        .expect("write generated cloud configuration");
    tauri_build::build()
}

fn config_value(
    environment_name: &str,
    property_name: &str,
    properties: &HashMap<String, String>,
) -> String {
    env::var(environment_name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| properties.get(property_name).cloned())
        .unwrap_or_default()
        .trim()
        .to_owned()
}

fn env_bool(name: &str, default: bool) -> bool {
    env::var(name)
        .ok()
        .map(|value| value.trim().eq_ignore_ascii_case("true"))
        .unwrap_or(default)
}

fn read_properties(path: &PathBuf) -> HashMap<String, String> {
    let Ok(contents) = fs::read_to_string(path) else {
        return HashMap::new();
    };
    contents
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            Some((key.trim().to_owned(), value.trim().to_owned()))
        })
        .collect()
}
