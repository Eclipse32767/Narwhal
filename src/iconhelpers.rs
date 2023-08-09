use std::collections::HashMap;
use freedesktop_icons::lookup;
use xdg_utils::query_mime_info;
const THEME: &str = "Adwaita";

pub fn clean_bad_mime(mime: String) -> String {
    let substrings_type: Vec<&str> = mime.split("-").collect();
    let category = substrings_type[0].to_string();
    if category == "application".to_string() {
        String::from("application-x-executable")
    } else if mime == "inode-directory" {
        String::from("folder")
    } else {
        format!("{}-x-generic", category)
    }
}
pub fn get_file_mimetype(path: String) -> String {
    let raw_data = match query_mime_info(path) {
        Ok(x) => x,
        Err(x) => panic!("{}", x)  
    };
    match std::str::from_utf8(&raw_data) {
        Ok(x) => x.to_string(),
        Err(e) => panic!("{}", e)
    }
}
pub async fn get_file_icon(cache: HashMap<String, String>, path: String) -> (Option<HashMap<String, String>>, String) {
    let icon_cache = cache.clone();
    let mut cache_changes: HashMap<String, String> = HashMap::new();
    let icon_out = icon_cache.get(&path);
    match icon_out {
        Some(icon) => match lookup(icon).with_cache().with_size(32).with_theme(THEME).find() {
            Some(x) => (None, x.to_string_lossy().to_string()),
            None => {
                let newicon = clean_bad_mime(icon.to_string());
                match lookup(&newicon).with_cache().with_size(32).with_theme(THEME).find() {
                    Some(x) => (None, x.to_string_lossy().to_string()),
                    None => (None, format!("{}/resources/text-rust.svg", env!("CARGO_MANIFEST_DIR")))
                }
            }
        }
        None => {
            let output = cacheless_get_file_icon(path.clone());
            cache_changes.insert(path, output.clone());
            (Some(cache_changes), lookup(&output).with_cache().with_size(32).with_theme(&THEME).find().unwrap().to_string_lossy().to_string())
        }
    }
}
pub fn cacheless_get_file_icon(path: String) -> String {
    let mut mimetype = get_file_mimetype(path.clone()).replace("/", "-");
    if mimetype == "inode-directory" {
        String::from("folder")
    } else {
        match lookup(&mimetype).with_cache().with_size(32).with_theme(&THEME).find() {
            Some(..) => mimetype,
            None => {
                mimetype = clean_bad_mime(mimetype);
                match lookup(&mimetype).with_cache().with_size(32).with_theme(&THEME).find() {
                    Some(..) => mimetype,
                    None => format!("text-x-generic")
                }
            }
        }
    }
}