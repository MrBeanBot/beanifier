//! Pure client-side WASM frontend for the beanifier.
//!
//! The entire UI runs in the browser as WebAssembly via [Yew](https://yew.rs) —
//! there is no server. Beanification happens locally on every keystroke by
//! calling straight into [`beanifier_core`], which compiles to `wasm32`
//! unchanged because it performs no I/O.

#![forbid(unsafe_code)]

use beanifier_core::{BeanConfig, Beanifier};
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

/// Inline stylesheet — bundled into the page so the app needs no asset server.
const STYLES: &str = r#"
:root { --bg:#1b140b; --panel:#2a1f11; --accent:#e0a80d; --ink:#f4ecdd; --muted:#b6a488; }
* { box-sizing: border-box; }
body { margin:0; font-family: 'Segoe UI', system-ui, sans-serif; background:var(--bg);
       color:var(--ink); line-height:1.5; }
header { padding:2rem 1rem 1rem; text-align:center; }
header h1 { margin:0; font-size:2.4rem; color:var(--accent); letter-spacing:.5px; }
header p { color:var(--muted); margin:.4rem 0 0; }
main { max-width:820px; margin:0 auto; padding:1rem; }
.panel { background:var(--panel); border:1px solid #3a2c17; border-radius:12px;
         padding:1.25rem; margin-bottom:1.25rem; }
label { display:block; font-weight:600; margin:.75rem 0 .3rem; }
textarea, input[type=number], input[type=text] { width:100%; padding:.6rem;
         border-radius:8px; border:1px solid #4a381d; background:#150f07; color:var(--ink);
         font-size:1rem; font-family:inherit; }
textarea { min-height:140px; resize:vertical; }
.controls { display:grid; grid-template-columns:repeat(auto-fit,minmax(150px,1fr)); gap:1rem; }
.check { display:flex; align-items:center; gap:.5rem; margin-top:1.4rem; }
.check input { width:auto; }
.output { white-space:pre-wrap; word-break:break-word; background:#150f07; border-radius:8px;
          padding:1rem; border:1px solid #4a381d; min-height:2rem; }
footer { text-align:center; color:var(--muted); padding:2rem 1rem; font-size:.85rem; }
code { background:#150f07; padding:.15rem .4rem; border-radius:5px; }
a { color:var(--accent); }
"#;

/// The whole application: a reactive form whose output re-beanifies live.
#[function_component(App)]
pub fn app() -> Html {
    let d = BeanConfig::default();
    let text = use_state(String::new);
    let seed = use_state(|| d.seed);
    let sig_freq = use_state(|| d.signature_frequency);
    let max_syllables = use_state(|| d.max_syllables);
    let preserve_case = use_state(|| d.preserve_case);

    let config = BeanConfig {
        seed: *seed,
        signature_frequency: *sig_freq,
        max_syllables: (*max_syllables).max(1),
        preserve_case: *preserve_case,
    };
    let output = Beanifier::new(config).beanify_text(&text);

    let on_text = {
        let text = text.clone();
        Callback::from(move |e: InputEvent| {
            let el: HtmlTextAreaElement = e.target_unchecked_into();
            text.set(el.value());
        })
    };
    let on_seed = {
        let seed = seed.clone();
        Callback::from(move |e: InputEvent| {
            let el: HtmlInputElement = e.target_unchecked_into();
            if let Ok(v) = el.value().parse() {
                seed.set(v);
            }
        })
    };
    let on_sig = {
        let sig_freq = sig_freq.clone();
        Callback::from(move |e: InputEvent| {
            let el: HtmlInputElement = e.target_unchecked_into();
            if let Ok(v) = el.value().parse() {
                sig_freq.set(v);
            }
        })
    };
    let on_max = {
        let max_syllables = max_syllables.clone();
        Callback::from(move |e: InputEvent| {
            let el: HtmlInputElement = e.target_unchecked_into();
            if let Ok(v) = el.value().parse() {
                max_syllables.set(v);
            }
        })
    };
    let on_case = {
        let preserve_case = preserve_case.clone();
        Callback::from(move |e: Event| {
            let el: HtmlInputElement = e.target_unchecked_into();
            preserve_case.set(el.checked());
        })
    };

    html! {
        <>
            <style>{ STYLES }</style>
            <header>
                <h1>{ "🫘 Beanifier" }</h1>
                <p>{ "Turn any text into glorious Mr-Bean-speak — entirely in your browser." }</p>
            </header>
            <main>
                <div class="panel">
                    <label for="text">{ "Your text" }</label>
                    <textarea id="text" placeholder="Type something sensible…"
                        oninput={on_text} value={(*text).clone()} />
                    <div class="controls">
                        <div>
                            <label for="seed">{ "Seed" }</label>
                            <input type="number" id="seed" value={seed.to_string()} oninput={on_seed} />
                        </div>
                        <div>
                            <label for="sig">{ "Signature freq." }</label>
                            <input type="number" step="0.01" min="0" max="1" id="sig"
                                value={sig_freq.to_string()} oninput={on_sig} />
                        </div>
                        <div>
                            <label for="max">{ "Max syllables" }</label>
                            <input type="number" min="1" id="max"
                                value={max_syllables.to_string()} oninput={on_max} />
                        </div>
                        <div class="check">
                            <input type="checkbox" id="case" checked={*preserve_case} onchange={on_case} />
                            <label for="case" style="margin:0">{ "Preserve case" }</label>
                        </div>
                    </div>
                </div>

                <div class="panel">
                    <label>{ "Bean says…" }</label>
                    <div class="output">{ output }</div>
                </div>
            </main>
            <footer>
                { "Bean-powered — 100% Rust, compiled to WebAssembly. No server involved." }
            </footer>
        </>
    }
}
