use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, DragEvent, Event, File, FormData, HtmlButtonElement, HtmlElement, HtmlFormElement, HtmlInputElement, Request, RequestInit, Response};

fn get_api_url() -> &'static str {
    option_env!("API_URL").unwrap_or("http://localhost:3000/upload")
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let document = window().unwrap().document().unwrap();
    let container = document.create_element("div")?.dyn_into::<HtmlElement>()?;
    container.set_class_name("container");
    let title = document.create_element("h2")?.dyn_into::<HtmlElement>()?;
    title.set_inner_html("File Upload");
    container.append_child(&title)?;

    let form = document.create_element("form")?.dyn_into::<HtmlFormElement>()?;
    let file_info = document.create_element("div")?.dyn_into::<HtmlElement>()?;
    file_info.set_class_name("file-info");
    file_info.set_inner_text("No file selected");
    form.append_child(&file_info)?;
    let file_drop = document.create_element("div")?.dyn_into::<HtmlElement>()?;
    file_drop.set_class_name("drag-drop-area");
    file_drop.set_inner_text("Drag & drop file or click here");

    let input: HtmlInputElement = document.create_element("input")?.dyn_into()?;
    input.set_type("file");
    input.set_class_name("file-input");
    file_drop.append_child(&input)?;
    form.append_child(&file_drop)?;

    let submit_button = document.create_element("button")?.dyn_into::<HtmlButtonElement>()?;
    submit_button.set_inner_text("Submit");
    submit_button.set_type("submit");
    form.append_child(&submit_button)?;
    container.append_child(&form)?;

    document.body().unwrap().append_child(&container)?;

    {
        let input = input.clone();
        let closure = Closure::wrap(Box::new(move |event: Event| {
            input.click();
        }) as Box<dyn FnMut(_)>);
        file_drop.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        
        let on_drop = {
            let file_info = file_info.clone();
            let input = input.clone();
            Closure::wrap(Box::new(move |event: DragEvent| {
                web_sys::console::log_1(&"Drop event".into());
                event.prevent_default();
                if let Some(files) = event.data_transfer().and_then(|d| d.files()) {
                    if files.length() > 0 {
                        input.set_files(Some(&files));
                        if let Some(file) = files.get(0) {
                            let name = file.name();
                            let size = file.size();
                            file_info.set_inner_text(&format!("📄 {} ({} bytes)", name, size));
                        }
                        web_sys::console::log_1(&"File dropped and selected!".into());
                    }
                }
            }) as Box<dyn FnMut(_)>)
        };


        let on_drag = {
            let file_drop = file_drop.clone();
            Closure::wrap(Box::new(move |event: DragEvent| {
                event.prevent_default();
                file_drop.class_list().add_1("drag-over").unwrap();
            }) as Box<dyn FnMut(_)>)
        };

        let on_drag_leave = {
            let file_drop = file_drop.clone();
            Closure::wrap(Box::new(move |_event: DragEvent| {
                file_drop.class_list().remove_1("drag-over").unwrap();
            }) as Box<dyn FnMut(_)>)
        };
        file_drop.add_event_listener_with_callback("drop", on_drop.as_ref().unchecked_ref())?;
        file_drop.add_event_listener_with_callback("dragover", on_drag.as_ref().unchecked_ref())?;
        file_drop.add_event_listener_with_callback("dragleave", on_drag_leave.as_ref().unchecked_ref())?;

        on_drop.forget();
        on_drag.forget();
        on_drag_leave.forget();
    }

    {
        let file_info = file_info.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
            web_sys::console::log_1(&"Changes".into());
            let input: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
            if let Some(file_list) = input.files() {
                if let Some(file) = file_list.get(0) {
                    web_sys::console::log_1(&file.name().into());
                    let name = file.name();
                    let size = file.size();
                    file_info.set_inner_text(&format!("📄 {} ({} bytes)", name, size));
                }
            }
        }) as Box<dyn FnMut(_)>);

        input.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        let on_submit = Closure::wrap(Box::new(move |event: Event| {
            event.prevent_default();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let _ = file_upload(file);
                } else {
                    show_alert("No file selected");
                }
            } else {
                    show_alert("No file selected");
            }
        }) as Box<dyn FnMut(_)>);

        form.add_event_listener_with_callback("submit", on_submit.as_ref().unchecked_ref())?;
        on_submit.forget();
    }
    Ok(())
}

fn file_upload(file: File) -> Result<(), JsValue> {
    let form_data = FormData::new()?;
    form_data.append_with_blob("file", &file)?;

    let mut opt = RequestInit::new();
    opt.set_method("POST");
    opt.set_body(&form_data);

    let request = Request::new_with_str_and_init(get_api_url(), &opt)?;
    request.headers().set("Accept", "application/json")?;

    let window = web_sys::window().unwrap();
    let request_promise = window.fetch_with_request(&request);

    spawn_local( async move {
        let res_val = wasm_bindgen_futures::JsFuture::from(request_promise).await.unwrap();
        let res: Response = res_val.dyn_into().unwrap();
        if res.ok() {
            show_alert(format!("Upload completed: {}", res.status()).as_str());
        } else {
            show_alert(format!("Upload failed: {}", res.status()).as_str());
        }
    });

    Ok(())
}

fn show_alert(message: &str) {
    window()
        .unwrap()
        .alert_with_message(message)
        .unwrap();
}