use genanki_rs_rev::{Deck, Field, Model, Note, Package, Template};
use js_sys::{Array, Uint8Array};
use leptos::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url, window};

const COMMON_LANGUAGES: &[(&str, &str)] = &[
    ("ar_SA", "🇸🇦 Arabic"),
    ("zh_CN", "🇨🇳 Chinese (Simplified)"),
    ("zh_TW", "🇹🇼 Chinese (Traditional)"),
    ("nl_NL", "🇳🇱 Dutch"),
    ("en_AU", "🇦🇺 English (Australia)"),
    ("en_GB", "🇬🇧 English (UK)"),
    ("en_US", "🇺🇸 English (US)"),
    ("fr_FR", "🇫🇷 French (France)"),
    ("de_DE", "🇩🇪 German (Germany)"),
    ("it_IT", "🇮🇹 Italian"),
    ("ja_JP", "🇯🇵 Japanese"),
    ("ko_KR", "🇰🇷 Korean"),
    ("pt_BR", "🇧🇷 Portuguese (Brazil)"),
    ("pt_PT", "🇵🇹 Portuguese (Portugal)"),
    ("ru_RU", "🇷🇺 Russian"),
    ("es_AR", "🇦🇷 Spanish (Argentina)"),
    ("es_MX", "🇲🇽 Spanish (Mexico)"),
    ("es_ES", "🇪🇸 Spanish (Spain)"),
];

const PREVIEW_PHRASE: &str = "Io sono un ragazzo";
const PREVIEW_WORD: &str = "ragazzo";
const PREVIEW_TRANSLATION: &str = "young man";

fn is_valid_iso_code(code: &str) -> bool {
    let mut parts = code.split(|c| c == '_' || c == '-');
    if let Some(lang) = parts.next() {
        if lang.len() < 2 || lang.len() > 3 || !lang.chars().all(|c| c.is_ascii_lowercase()) {
            return false;
        }
    } else {
        return false;
    }
    if let Some(region) = parts.next() {
        if region.len() < 2
            || region.len() > 3
            || !region
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            return false;
        }
    }
    parts.next().is_none()
}

fn random_id() -> i64 {
    (js_sys::Math::random() * (1_000_000_000_f64)) as i64
}

fn download_bytes(bytes: &[u8], filename: &str, mime_type: &str) -> Result<(), JsValue> {
    let uint8_array = Uint8Array::new_with_length(bytes.len() as u32);
    uint8_array.copy_from(bytes);

    let array = Array::new();
    array.push(&uint8_array);

    let options = BlobPropertyBag::new();
    options.set_type(mime_type);
    let blob = Blob::new_with_u8_array_sequence_and_options(&array, &options)?;

    let url = Url::create_object_url_with_blob(&blob)?;
    let window = window().ok_or_else(|| JsValue::from_str("No window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("No document"))?;

    let a = document
        .create_element("a")?
        .dyn_into::<HtmlAnchorElement>()?;
    a.set_href(&url);
    a.set_download(filename);
    a.click();

    Url::revoke_object_url(&url)?;
    Ok(())
}

#[component]
fn App() -> impl IntoView {
    let (your_lang, set_your_lang) = signal(String::from("en_US"));
    let (target_lang, set_target_lang) = signal(String::from("it_IT"));
    let (highlight_color, set_highlight_color) = signal(String::from("#ff0000"));
    let (custom_your_lang, set_custom_your_lang) = signal(false);
    let (custom_target_lang, set_custom_target_lang) = signal(false);
    let (enable_active, set_enable_active) = signal(true);
    let (enable_passive, set_enable_passive) = signal(true);
    let (enable_writing, set_enable_writing) = signal(true);

    let (status_msg, set_status_msg) = signal(Option::<(String, bool)>::None);

    let passive_audio_front = NodeRef::<leptos::html::Audio>::new();
    let passive_audio_back = NodeRef::<leptos::html::Audio>::new();
    let active_audio_front = NodeRef::<leptos::html::Audio>::new();
    let active_audio_back = NodeRef::<leptos::html::Audio>::new();

    let generate_deck = move |_| {
        if !enable_active.get() && !enable_passive.get() && !enable_writing.get() {
            set_status_msg.set(Some((
                "Please select at least one card type to generate.".to_string(),
                true,
            )));
            return;
        }

        let current_your_lang = your_lang.get();
        let current_target_lang = target_lang.get();

        if !is_valid_iso_code(&current_your_lang) {
            set_status_msg.set(Some((
                format!(
                    "Invalid native language code: '{}'. Use formats like 'en' or 'en_US'.",
                    current_your_lang
                ),
                true,
            )));
            return;
        }

        if !is_valid_iso_code(&current_target_lang) {
            set_status_msg.set(Some((
                format!(
                    "Invalid target language code: '{}'. Use formats like 'es' or 'es_ES'.",
                    current_target_lang
                ),
                true,
            )));
            return;
        }

        let current_name = format!(
            "AnkiPolyglot({}-{})",
            current_your_lang, current_target_lang
        );
        let current_highlight = highlight_color.get();

        let model_id = random_id();
        let deck_id = random_id();

        let fields = vec![
            Field::new(&format!("🔤 Text ({current_target_lang})")),
            Field::new(&format!("🔄 Translation ({current_your_lang})")),
            Field::new(&format!("❓ New word ({current_target_lang})")),
            Field::new("🗣️ Pronunciation"),
        ];

        let mut templates = Vec::new();

        let mut add_template = |name: &'static str| {
            let (front_html, back_html) = match name {
                "Active" => (
                    include_str!("../cardtypes/Active/Front.html"),
                    include_str!("../cardtypes/Active/Back.html"),
                ),
                "Passive" => (
                    include_str!("../cardtypes/Passive/Front.html"),
                    include_str!("../cardtypes/Passive/Back.html"),
                ),
                "Writing" => (
                    include_str!("../cardtypes/Writing/Front.html"),
                    include_str!("../cardtypes/Writing/Back.html"),
                ),
                _ => panic!("Unknown template: {}", name),
            };
            let qfmt_processed = front_html
                .replace("target_lang", &current_target_lang)
                .replace("your_lang", &current_your_lang)
                .replace("highlight_color", &current_highlight);
            let afmt_processed = back_html
                .replace("target_lang", &current_target_lang)
                .replace("your_lang", &current_your_lang)
                .replace("highlight_color", &current_highlight);
            templates.push(
                Template::new(name)
                    .qfmt(&qfmt_processed)
                    .afmt(&afmt_processed),
            );
        };
        if enable_passive.get() {
            add_template("Passive");
        }
        if enable_active.get() {
            add_template("Active");
        }
        if enable_writing.get() {
            add_template("Writing");
        }

        let model = Model::new(model_id, &current_name, fields, templates).css(
            ".card {
                font-family: arial;
                font-size: 20px;
                line-height: 1.5;
                text-align: center;
                color: black;
                background-color: white;
            }
            ",
        );
        let mut deck = Deck::new(deck_id, &current_name, "");

        deck.add_note(
            Note::new(
                model,
                vec![
                    &format!("Phrase in {}", current_target_lang),
                    &format!("Translation in {}", current_your_lang),
                    &format!("New Word in {}", current_target_lang),
                    "IPA pronunciation",
                ],
            )
            .unwrap(),
        );

        let package = Package::new(vec![deck], std::collections::HashMap::new()).unwrap();
        let mut raw_bytes = std::io::Cursor::new(Vec::new());
        package.write(&mut raw_bytes).unwrap();
        let final_bytes = raw_bytes.into_inner();
        let file_name = format!("{}.apkg", current_name);
        match download_bytes(&final_bytes, &file_name, "application/octet-stream") {
            Ok(_) => {
                set_status_msg.set(Some((
                    format!(
                        "Deck '{}' generated and downloaded successfully!",
                        current_name
                    ),
                    false,
                )));
            }
            Err(e) => {
                set_status_msg.set(Some((format!("Error downloading deck: {:?}", e), true)));
            }
        }

        set_status_msg.set(Some((
            format!("Deck '{}' generated successfully!", current_name),
            false,
        )));
    };

    let render_front_preview = move || {
        let color = highlight_color.get();
        let phrase = PREVIEW_PHRASE;
        let word = PREVIEW_WORD;

        if phrase.contains(&word) && !word.is_empty() {
            phrase.replace(&word, &format!("<span style='color: {}; font-weight: bold; text-decoration: underline;'>{}</span>", color, word))
        } else {
            phrase.to_string()
        }
    };

    view! {
        <div class="app-container">
            <div class="content-wrapper">
                <div class="config-panel">
                    <div class="panel-header">
                        <img src="anki-icon.svg" alt="Anki Icon" width=100 height=100 />
                        <div style="flex-direction: column">
                            <h2> <span style="color: #27a1ed"> "Anki" </span>"Polyglot"</h2>
                            <h4>"A language learning template" <br /> "(with TTS and IPA 🗣️)"</h4>
                            </div>
                    </div>


                    <div class="input-group">
                        <span class="dynamic-direction"></span>
                        <div class="checkbox-group">
                            <label><input type="checkbox" prop:checked=move || enable_passive.get() on:change=move |_| { set_enable_passive.set(!enable_passive.get()); set_status_msg.set(None); } /> "Passive"</label>
                            <label><input type="checkbox" prop:checked=move || enable_active.get() on:change=move |_| { set_enable_active.set(!enable_active.get()); set_status_msg.set(None); } /> "Active"</label>
                            <label><input type="checkbox" prop:checked=move || enable_writing.get() on:change=move |_| { set_enable_writing.set(!enable_writing.get()); set_status_msg.set(None); } /> "Writing"</label>
                        </div>
                    </div>

                    <label class="input-group">
                        <div class="space-between">
                            <span>"YOUR NATIVE LANGUAGE (mother tongue)"</span>
                        </div>
                        <select
                            class="text-input"
                            style:display=move || if custom_your_lang.get() { "none" } else { "block" }
                            prop:value=your_lang
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                if val == "custom" {
                                    set_custom_your_lang.set(true);
                                    set_your_lang.set(String::new());
                                } else {
                                    set_your_lang.set(val);
                                }
                                set_status_msg.set(None);
                            }
                        >
                            {COMMON_LANGUAGES.iter().map(|(iso, name)| {
                                view! { <option value=*iso>{*name}</option> }
                            }).collect::<Vec<_>>()}

                            <option disabled> "──────────" </option>
                            <option value="custom">"Not in list? Click here"</option>
                        </select>


                        <div style:display=move || if custom_your_lang.get() { "flex" } else { "none" } style="gap: 8px;">
                            <input
                                type="text"
                                placeholder="(example: en_US) 4-letter ISO code"
                                class="text-input"
                                style="flex: 1;"
                                prop:value=your_lang
                                on:input=move |ev| {
                                    set_your_lang.set(event_target_value(&ev));
                                    set_status_msg.set(None);
                                }
                            />
                            <button
                                type="button"
                                class="toggle-btn"
                                style="padding: 0 12px; cursor: pointer;"
                                on:click=move |_| {
                                    set_custom_your_lang.set(false);
                                    set_your_lang.set("en_US".to_string());
                                }
                            >
                                "❌ Cancel"
                            </button>
                        </div>
                    </label>


                    <label class="input-group">
                        <div class="space-between">
                            <span>"TARGET LANGUAGE (language to learn)"</span>
                        </div>
                        <select
                            class="text-input"
                            style:display=move || if custom_target_lang.get() { "none" } else { "block" }
                            prop:value=target_lang
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                if val == "custom" {
                                    set_custom_target_lang.set(true);
                                    set_target_lang.set(String::new());
                                } else {
                                    set_target_lang.set(val);
                                }
                                set_status_msg.set(None);
                            }
                        >
                            {COMMON_LANGUAGES.iter().map(|(iso, name)| {
                                view! { <option value=*iso>{*name}</option> }
                            }).collect::<Vec<_>>()}

                            <option disabled> "──────────" </option>
                            <option value="custom">"Not in list? Click here"</option>
                        </select>

                        <div style:display=move || if custom_target_lang.get() { "flex" } else { "none" } style="gap: 8px;">
                            <input
                                type="text"
                                placeholder="(example: es_ES) 4-letter ISO code"
                                class="text-input"
                                style="flex: 1;"
                                prop:value=target_lang
                                on:input=move |ev| {
                                    set_target_lang.set(event_target_value(&ev));
                                    set_status_msg.set(None);
                                }
                            />
                            <button
                                type="button"
                                class="toggle-btn"
                                style="padding: 0 12px; cursor: pointer;"
                                on:click=move |_| {
                                    set_custom_target_lang.set(false);
                                    set_target_lang.set("es_ES".to_string());
                                }
                            >
                                "❌ Cancel"
                            </button>
                        </div>
                    </label>

                    <label class="input-group">
                        "HIGHLIGHT COLOR (use to highlight new words)"
                        <input type="color" class="color-picker" prop:value=highlight_color on:input=move |ev| set_highlight_color.set(event_target_value(&ev)) />
                    </label>

                    {move || status_msg.get().map(|(msg, is_error)| {
                        let status_class = if is_error { "status-error" } else { "status-success" };
                        view! {
                            <div class=format!("status-banner {}", status_class)>
                                {if is_error { "⚠️ " } else { "✅ " }} {msg}
                            </div>
                        }
                    })}

                    <button class="primary-btn" on:click=generate_deck>
                        "Download .apkg file"
                    </button>
                    <span style="font-family: monospace;">
                        {move || format!("A new AnkiPolyglot ({}-{}) example deck and note type will be imported, test it if the voices are working and then", your_lang.get(), target_lang.get())}
                        <span style="font-family: monospace; color: #ff0000">
                            " YOU CAN DELETE THE DECK"
                        </span>
                    </span>
                </div>
                <div class="preview-panel">
                    <div class="preview-header">
                        <h2>"Preview Example"</h2>

                        <p>"See how an American would use this template for learning Italian"</p>
                    </div>


                    {move || enable_passive.get().then(|| {
                        view! {
                            <div class="card-container">
                                <div style="display: flex; align-items: baseline; gap: 8px;">
                                    <div class="card-label color-passive">
                                        "📖 Passive Card"
                                    </div>
                                    <span style="font-size: 0.7em; opacity: 0.8;">
                                        "(passively consuming Italian)"
                                    </span>
                                </div>
                                <div class="card-face">
                                    <div class="face-label">"FRONT"</div>
                                    <div class="audio-box">
                                        <p class="face-content" inner_html=render_front_preview style="margin: 0;" />
                                        <button
                                            on:click=move |_| {
                                                if let Some(audio) = passive_audio_front.get() {
                                                    let _ = audio.play();
                                                }
                                            }
                                            style="background: transparent; border: none; cursor: pointer; padding: 0; display: flex; align-items: center; justify-content: center;"
                                            title="Play Audio"
                                        >
                                            <svg width="24" height="24" viewBox="0 0 24 24" fill="#ffffff" xmlns="http://www.w3.org/2000/svg">
                                                <path d="M8 5v14l11-7z" />
                                            </svg>
                                        </button>
                                        <audio node_ref=passive_audio_front src="io_sono_un_ragazzo.mp3" style="display: none;"></audio>
                                    </div>
                                </div>
                                <div class="card-face">
                                    <div class="face-label">"BACK"</div>
                                    <div class="face-content audio-box">
                                        <span>
                                            <span style:color=move || highlight_color.get() class="word-highlight">
                                                {PREVIEW_WORD}
                                            </span>
                                            "(raˈɡat.t͡so) = " {PREVIEW_TRANSLATION}
                                        </span>

                                        <button
                                            on:click=move |_| {
                                                if let Some(audio) = passive_audio_back.get() {
                                                    let _ = audio.play();
                                                }
                                            }
                                            style="background: transparent; border: none; cursor: pointer; padding: 0; display: flex; align-items: center; justify-content: center;"
                                            title="Play Audio"
                                        >
                                            <svg width="24" height="24" viewBox="0 0 24 24" fill="#ffffff" xmlns="http://www.w3.org/2000/svg">
                                                <path d="M8 5v14l11-7z" />
                                            </svg>
                                        </button>
                                        <audio node_ref=passive_audio_back src="ragazzo.mp3" style="display: none;"></audio>

                                    </div>
                                </div>
                            </div>
                        }
                    })}

                    {move || enable_active.get().then(|| {
                        view! {
                            <div class="card-container">
                                 <div style="display: flex; align-items: baseline; gap: 8px;">
                                    <div class="card-label color-active">"🗣️ Active Card"</div>
                                    <span style="font-size: 0.7em; opacity: 0.8;">
                                        "(actively producing Italian)"
                                    </span>
                                 </div>


                                <div class="card-face">
                                    <div class="face-label">"FRONT"</div>
                                    <p class="face-content audio-box">
                                    {PREVIEW_TRANSLATION}

                                    <button
                                        on:click=move |_| {
                                            if let Some(audio) = active_audio_front.get() {
                                                let _ = audio.play();
                                            }
                                        }
                                        style="background: transparent; border: none; cursor: pointer; padding: 0; display: flex; align-items: center; justify-content: center;"
                                        title="Play Audio"
                                    >
                                        <svg width="24" height="24" viewBox="0 0 24 24" fill="#ffffff" xmlns="http://www.w3.org/2000/svg">
                                            <path d="M8 5v14l11-7z" />
                                        </svg>
                                    </button>
                                    <audio node_ref=active_audio_front src="young_man.mp3" style="display: none;"></audio>


                                    </p>

                                </div>
                                <div class="card-face">
                                    <div class="face-label">"BACK"</div>
                                    <div class="audio-box">
                                        <p class="face-content" inner_html=render_front_preview />
                                        "(raˈɡat.t͡so)"
                                        <button
                                            on:click=move |_| {
                                                if let Some(audio) = active_audio_back.get() {
                                                    let _ = audio.play();
                                                }
                                            }
                                            style="background: transparent; border: none; cursor: pointer; padding: 0; display: flex; align-items: center; justify-content: center;"
                                            title="Play Audio"
                                        >
                                            <svg width="24" height="24" viewBox="0 0 24 24" fill="#ffffff" xmlns="http://www.w3.org/2000/svg">
                                                <path d="M8 5v14l11-7z" />
                                            </svg>
                                        </button>
                                        <audio node_ref=active_audio_back src="io_sono_un_ragazzo.mp3" style="display: none;"></audio>

                                    </div>
                                </div>
                            </div>
                        }
                    })}

                    {move || enable_writing.get().then(|| {
                        view! {
                            <div class="card-container">
                                <div style="display: flex; align-items: baseline; gap: 8px;">
                                    <div class="card-label color-writing">"✍️ Writing Card"</div>
                                    <span style="font-size: 0.7em; opacity: 0.8;">
                                        "(improving spelling and active recall)"
                                    </span>
                                </div>
                                <div class="card-face">
                                    <div class="face-label">"FRONT"</div>
                                    <p class="face-content">{PREVIEW_TRANSLATION}</p>
                                    <div class="text-center">
                                        <div class="type-box">"ragazoo"</div>
                                    </div>
                                </div>
                                <div class="card-face">
                                    <div class="face-label">"BACK"</div>
                                    <div class="text-center margin-bottom-8">
                                        <div class="correct-answer">
                                            <span style="color: #00ff00">"ragaz"</span>
                                            <span style="color: #ff0000">"z"</span>
                                            <span style="color: #00ff00">"o"</span>

                                        </div>
                                    </div>
                                </div>
                            </div>
                        }
                    })}
                    <p>
                        "This website is powered by the "
                        <a
                            href="https://github.com/viniciusdutra314/genanki-wasm"
                            target="_blank"
                            rel="noopener noreferrer"
                            style="color: #27a1ed; text-decoration: underline;"
                        >
                            "genanki-wasm"
                        </a>
                        " library"
                    </p>
                </div>
            </div>
        </div>
        <footer>
            <div style="display: flex; align-items: center; justify-content: center; color: #ffffff; gap: 16px;">

            </div>
        </footer>
    }
}

fn main() {
    leptos::mount::mount_to_body(App)
}
