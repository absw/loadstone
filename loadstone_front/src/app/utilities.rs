use wasm_bindgen::{prelude::*, JsCast};

pub fn download_file(name: &str, data: &str) -> Result<(), JsValue> {
    use web_sys::{Blob, BlobPropertyBag, HtmlElement, Url};
    let document = web_sys::window().unwrap().document().unwrap();

    let mut props = BlobPropertyBag::new();
    props.type_("text/plain");

    let blob =
        Blob::new_with_str_sequence_and_options(&JsValue::from_serde(&[data]).unwrap(), &props)?;
    let link = document.create_element("a")?.dyn_into::<HtmlElement>()?;
    link.set_attribute("href", Url::create_object_url_with_blob(&blob)?.as_str())?;
    link.set_attribute("download", name)?;

    let body = document.body().unwrap();
    body.append_child(&link)?;
    link.click();
    body.remove_child(&link)?;

    Ok(())
}
