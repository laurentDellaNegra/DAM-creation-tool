#[cfg(not(target_arch = "wasm32"))]
pub fn download_payload(payload: &dam_core::SubmissionPayload) -> Result<String, String> {
    std::fs::write(&payload.filename, &payload.body).map_err(|error| error.to_string())?;
    Ok(payload.filename.clone())
}

#[cfg(target_arch = "wasm32")]
pub fn download_payload(payload: &dam_core::SubmissionPayload) -> Result<String, String> {
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
    Ok(payload.filename.clone())
}
