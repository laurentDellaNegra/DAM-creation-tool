#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub enum DownloadOutcome {
    Saved(String),
    Cancelled,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn download_payload(payload: &dam_core::AixmPayload) -> Result<DownloadOutcome, String> {
    let Some(path) = save_path(&payload.filename) else {
        return Ok(DownloadOutcome::Cancelled);
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("{}: {error}", parent.display()))?;
    }

    std::fs::write(&path, &payload.body).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(DownloadOutcome::Saved(path.display().to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
fn save_path(filename: &str) -> Option<std::path::PathBuf> {
    let mut dialog = rfd::FileDialog::new()
        .set_title("Save AIXM XML")
        .add_filter("XML", &["xml"])
        .set_file_name(filename);

    if let Some(downloads_dir) = downloads_dir() {
        dialog = dialog.set_directory(downloads_dir);
    }

    dialog.save_file()
}

#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
fn downloads_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("USERPROFILE")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            let drive = std::env::var_os("HOMEDRIVE")?;
            let path = std::env::var_os("HOMEPATH")?;
            Some(std::path::PathBuf::from(format!(
                "{}{}",
                drive.to_string_lossy(),
                path.to_string_lossy()
            )))
        })
        .map(|home| home.join("Downloads"))
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "windows")))]
fn downloads_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .map(|home| home.join("Downloads"))
}

#[cfg(target_arch = "wasm32")]
pub fn download_payload(payload: &dam_core::AixmPayload) -> Result<DownloadOutcome, String> {
    use wasm_bindgen::JsCast as _;

    let array = js_sys::Array::new();
    array.push(&wasm_bindgen::JsValue::from_str(&payload.body));

    let options = web_sys::BlobPropertyBag::new();
    options.set_type(payload.content_type);

    let blob = web_sys::Blob::new_with_str_sequence_and_options(&array, &options)
        .map_err(|error| format!("{error:?}"))?;
    let url =
        web_sys::Url::create_object_url_with_blob(&blob).map_err(|error| format!("{error:?}"))?;

    let window = web_sys::window().ok_or_else(|| "window is not available".to_owned())?;
    let document = window
        .document()
        .ok_or_else(|| "document is not available".to_owned())?;
    let anchor = document
        .create_element("a")
        .map_err(|error| format!("{error:?}"))?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|_| "created element is not an anchor".to_owned())?;

    anchor.set_href(&url);
    anchor.set_download(&payload.filename);
    anchor.click();

    let _ = web_sys::Url::revoke_object_url(&url);
    Ok(DownloadOutcome::Saved(payload.filename.clone()))
}
