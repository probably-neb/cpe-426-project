// use aes::Aes128;
use anyhow::Result;
use axum::{
    response::Html,
    routing::{get, post},
    Form, Router,
};
use formatx::*;

type Partial = Html<&'static str>;

const KEY_LEN: usize = 16;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(render(Index(KeyForm(None, None)))))
        .route("/key", post(key))
        .route("/message", post(message));

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(serde::Deserialize)]
struct KeyInput {
    pub key: String,
}

async fn key(Form(input): Form<KeyInput>) -> Partial {
    let key = &input.key;
    match validate_key(key) {
        Err(e) => {
            println!("Error: {:?}", e);
            render(KeyForm(Some(key), Some(&e.to_string())))
        }
        Ok(key) => render(format!("{}{}", KeyForm(Some(key), None), MessageForm(None))),
    }
}

fn validate_key(key: &str) -> Result<&str> {
    if key.len() != KEY_LEN {
        Err(anyhow::Error::msg(format!(
            "Key was {} character long but it must be 16 characters long",
            key.len()
        )))
    } else {
        Ok(key)
    }
}

#[derive(serde::Deserialize)]
struct MessageInput {
    message: String,
}

async fn message(Form(input): Form<MessageInput>) -> Partial {
    render(MessageForm(Some(&input.message)))
}

type TemplateResult = String;

fn render(templ: TemplateResult) -> Partial {
    return Html(templ.leak());
}

fn Index(children: TemplateResult) -> TemplateResult {
    formatx!(
        r#"<html>
        <head>
            <title>Basys3 AES Server</title>
            <meta name="viewport" content="width=device-width, initial-scale=1" />
            <meta charset="utf-8" />
            <script src="https://unpkg.com/htmx.org@1.9.6"></script>
            <script src="https://cdn.tailwindcss.com"></script>
        </head>
        <body class="px-10 py-10">
            {}
        </body>
    </html>"#,
        children
    )
    .unwrap()
}

fn KeyForm(value: Option<&str>, err: Option<&str>) -> TemplateResult {
    let value = value.unwrap_or("");
    let err = err.unwrap_or("");

    formatx!(
        r#"
        <form hx-post="/key" hx-swap="outerHTML" >
            <div class="flex flex-row gap-2">
            <label for="key">Enter your secret Key</label>
            <input type="text" class="border-2" name="key" value="{}"></input>
            <button type="submit" class="border-2 bg-slate-100">
                Submit
            </button>
            </div>
            <p style="color: #FF0000; font-size: 14px; font-weight: bold; margin-top: 5px;">{}</p>
        </form>
        <form id="message-form" hx-swap-oob="true"></form>
        "#,
        value,
        err
    )
    .unwrap()
}

fn MessageForm(message: Option<&str>) -> TemplateResult {
    let message = message.unwrap_or("");
    formatx!(
        r#"
        <form hx-post="/message" hx-swap="outerHTML" id="message-form" hx-swap-oob="true" class="flex flex-col gap-2">
             <label for="message">Enter Text:</label>
             <textarea type="text" id="message" name="message" class="w-[300px] border-2" rows="4" value="{}"></textarea>
             <button type="submit" class="border-2 bg-slate-100">
                Encrypt!
             </button>
        </form>
        "#,
        message
    )
    .unwrap()
}
