// use aes::Aes128;
use aes::cipher::{generic_array::GenericArray, BlockEncrypt, KeyInit};
use aes::Aes128;
use anyhow::{anyhow, Result};
use axum::{
    http::HeaderMap,
    response::Html,
    routing::{get, post},
    Form, Router,
};
use formatx::*;
use len_trait::Len;
use rand::Rng;

type Partial = Html<&'static str>;

const KEY_LEN: usize = 16;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(render(index())))
        .route("/submit", post(handle_submit))
        .route("/key", post(handle_set_key))
        .route("/encrypt", post(handle_encrypt_message))
        .route("/key/random", get(handle_random_key))
        .route("/message/random", get(handle_random_message));

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(serde::Deserialize, Debug)]
pub struct PageFormOpts {
    pub key: Option<String>,
    pub key_err: Option<String>,
    pub message: Option<String>,
    pub ciphertext: Option<String>,
}

impl PageFormOpts {
    fn clean(self) -> Self {
        Self {
            key: none_if_empty(self.key),
            key_err: none_if_empty(self.key_err),
            message: none_if_empty(self.message),
            ciphertext: none_if_empty(self.ciphertext),
        }
    }
}

impl Default for PageFormOpts {
    fn default() -> Self {
        return PageFormOpts {
            key: None,
            key_err: None,
            message: None,
            ciphertext: None,
        };
    }
}

fn index() -> TemplateResult {
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
        page_form(PageFormOpts::default())
    )
    .unwrap()
}

async fn handle_submit(headers: HeaderMap, Form(opts): Form<PageFormOpts>) -> Partial {
    // let PageFormOpts { key, key_err, message } = opts;
    dbg!("{} {}", &headers, &opts);
    render(page_form(opts))
}

async fn handle_random_key() -> Partial {
    let key = gen_random_key().into();

    return render(format!("{}{}", key_input(Some(key)), key_error(None, true)));
}

async fn handle_random_message() -> Partial {
    let text = gen_random_message();
    return Html(text.leak());
}

async fn handle_set_key(Form(opts): Form<PageFormOpts>) -> Partial {
    let PageFormOpts {
        key,
        message,
        key_err: _,
        ciphertext,
    } = opts;

    let (key, key_err) = match validate_key(key.clone()) {
        Ok(key) => (Some(key), None),
        Err(err) => (key, Some(err.to_string())),
    };
    render(page_form(PageFormOpts {
        key,
        key_err,
        message,
        ciphertext,
    }))
}

async fn handle_encrypt_message(Form(opts): Form<PageFormOpts>) -> Partial {
    let mut opts = opts.clean();

    let key = match validate_key(opts.key.clone()) {
        Ok(key) => key,
        Err(err) => {
            opts.key_err = Some(err.to_string());
            return render(page_form(opts));
        }
    };
    opts.ciphertext = Some(encrypt_string(
        &key,
        opts.message.as_ref().unwrap(),
    ));

    return render(page_form(opts));
}

type TemplateResult = String;

fn render(templ: TemplateResult) -> Partial {
    return Html(templ.leak());
}

fn page_form(
    PageFormOpts {
        key,
        key_err,
        message,
        ciphertext,
    }: PageFormOpts,
) -> TemplateResult {
    formatx!(
        r#"
            <form hx-post="/submit" hx-swap="outerHTML" id="form">
                {}
                {}
                {}
            </form>
        "#,
        key_form_group(key, key_err),
        message_form_group(message),
        cipher_form_group(ciphertext)
    )
    .unwrap()
}

fn key_form_group(key: Option<String>, key_err: Option<String>) -> TemplateResult {
    format!(
        r##"
            <div id="key-part" class="flex flex-row gap-2">
                <label for="key">Secret Key</label>
                {}
                <button hx-post="/key" hx-target="#form" class="border-2 bg-slate-100">
                    Set
                </button>
                <button class="border-2 bg-slate-100" hx-get="/key/random" hx-target="#key-input">
                    Random Key
                </button>
            </div>
            {}
        "##,
        key_input(key),
        key_error(key_err, false)
    )
}

fn key_error(err: Option<String>, out_of_band: bool) -> TemplateResult {
    dbg!("{}", true.to_string());
    let err = err.unwrap_or_default();
    let oob = if out_of_band {
        r#"hx-swap-oob="true""#
    } else {
        ""
    };
    format!(
        r#"
            <p id="key-error" hx-swap-oob="{}" style="color: #FF0000; font-size: 14px; font-weight: bold; margin-top: 5px;">{}</p>
    "#,
        oob, err
    )
}

fn key_input(key: Option<String>) -> TemplateResult {
    let key = key.unwrap_or_default();
    format!(
        r#"
        <input spellcheck="false" type="text" id="key-input" class="border-2" name="key" value="{}"></input>
    "#,
        key
    )
}

fn message_form_group(message: Option<String>) -> TemplateResult {
    let message = message.unwrap_or_default();
    formatx!(
        r##"
            <div id="message-part" class="flex flex-col gap-2 py-4">
                <label for="message">Plain Text:</label>
                <textarea spellcheck="false" type="text" id="message" name="message" class="w-[600px] border-2" rows="4" >{}</textarea>
                <div class="flex flex-row justify-start gap-2">
                   <button class="border-2 bg-slate-100" hx-get="/message/random" hx-target="#message" hx-swap="innerHTML">
                       Random Message
                   </button>
                   <button hx-post="/encrypt" hx-target="form" class="border-2 bg-slate-100">
                      Encrypt!
                   </button>
                </div>
            </div>
        "##,
        message
    ).unwrap()
}

fn cipher_form_group(ct: Option<String>) -> TemplateResult {
    let ct = ct.unwrap_or_default();
    format!(
        r##"
            <div id="cipher-part" class="flex flex-col gap-2 py-2">
                <p>Cipher Text</p>
                <textarea readonly class="w-[600px] h-[200px] border-2 break-words">{}</textarea>
                <button hx-post="/encrypt" hx-target="form" class="block border-2 bg-slate-100">
                   Decrypt (TODO!)
                </button>
            </div>
        "##,
        ct
    )
}

fn validate_key<S>(key: Option<S>) -> Result<S>
where
    S: Len,
{
    let key = none_if_empty(key);
    match key {
        Some(key) => match key.len() {
            KEY_LEN => Ok(key),
            _ => Err(anyhow!(
                "Key is {} characters long but it must be 16 characters long",
                key.len()
            )),
        },
        None => Err(anyhow!("Key is required")),
    }
}

fn gen_random_key() -> String {
    let charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$^&*-=+?";
    return random_string::generate(KEY_LEN, charset);
}

fn gen_random_message() -> String {
    let mut rng = rand::thread_rng();
    let words: Vec<&str> = LOREM_IPSUM.split_whitespace().collect();
    let num_words = rng.gen_range(1..words.len());
    let words: Vec<&str> = rng
        .sample_iter(rand::distributions::Slice::new(&words).unwrap())
        .take(num_words)
        .map(|s| *s)
        .collect();
    return words.join(" ").into();
}

fn encrypt_string(key: &str, pt: &str) -> String {
    type Block = GenericArray<u8, aes::cipher::typenum::U16>;

    let mut key_arr = [0; KEY_LEN];
    for (i, b) in key.bytes().enumerate() {
        key_arr[i] = b;
    }
    let key = GenericArray::from(key_arr);

    let num_blocks = (pt.len() + KEY_LEN) / KEY_LEN;
    let mut blocks = vec![Block::default(); num_blocks];
    let pt_bytes = pt.as_bytes();

    for i in (0..pt.len()).step_by(KEY_LEN) {
        let bytes = &pt_bytes[i..usize::min(i + KEY_LEN, pt.len() - 1)];
        let block = &mut blocks[i / KEY_LEN];

        for b in 0..bytes.len() {
            dbg!("{} {} {}", b, block.len(), bytes.len());
            block[b] = bytes[b]
        }
    }

    let cipher = Aes128::new(&key);
    cipher.encrypt_blocks(&mut blocks);

    let ct: Vec<u8> = blocks.into_iter().flatten().collect();
    return hex::encode_upper(ct);
}

fn none_if_empty<S>(s: Option<S>) -> Option<S>
where
    S: Len,
{
    s.and_then(|s| if s.len() == 0 { None } else { Some(s) })
}

const LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. A erat nam at lectus urna duis convallis convallis. Ut placerat orci nulla pellentesque dignissim enim sit amet venenatis. Interdum consectetur libero id faucibus nisl tincidunt eget nullam non. Diam vulputate ut pharetra sit amet aliquam id. Nisi lacus sed viverra tellus in hac habitasse platea dictumst. Mi proin sed libero enim sed faucibus turpis in eu. Vulputate odio ut enim blandit volutpat maecenas volutpat. Nulla facilisi nullam vehicula ipsum a. Tellus in hac habitasse platea dictumst vestibulum rhoncus est pellentesque. In tellus integer feugiat scelerisque varius morbi enim nunc faucibus. Auctor urna nunc id cursus metus aliquam eleifend mi. In hac habitasse platea dictumst vestibulum. Adipiscing bibendum est ultricies integer quis auctor. Risus quis varius quam quisque id. Nisl condimentum id venenatis a. Vitae sapien pellentesque habitant morbi tristique senectus et netus et. Ultrices gravida dictum fusce ut placerat orci nulla pellentesque dignissim. Ullamcorper morbi tincidunt ornare massa eget egestas purus. Amet massa vitae tortor condimentum. Tristique et egestas quis ipsum. Pulvinar mattis nunc sed blandit libero volutpat. Interdum velit euismod in pellentesque massa placerat. Tellus elementum sagittis vitae et leo duis ut diam. Nisl rhoncus mattis rhoncus urna neque viverra justo nec. Arcu ac tortor dignissim convallis aenean et tortor. Faucibus interdum posuere lorem ipsum dolor. At ultrices mi tempus imperdiet. Velit sed ullamcorper morbi tincidunt. Sed viverra ipsum nunc aliquet bibendum enim facilisis gravida. Mauris pellentesque pulvinar pellentesque habitant morbi tristique senectus et. Nibh sit amet commodo nulla facilisi nullam vehicula ipsum. Tristique nulla aliquet enim tortor at auctor urna nunc. Massa vitae tortor condimentum lacinia quis vel eros donec. Lorem sed risus ultricies tristique nulla aliquet enim. Maecenas volutpat blandit aliquam etiam erat velit scelerisque in dictum. Cras tincidunt lobortis feugiat vivamus at augue eget arcu. Ultricies mi eget mauris pharetra et ultrices neque ornare. Diam quis enim lobortis scelerisque fermentum dui faucibus in ornare. Molestie ac feugiat sed lectus vestibulum mattis. Enim sed faucibus turpis in. Lectus urna duis convallis convallis tellus id. Cursus risus at ultrices mi tempus imperdiet nulla malesuada. Libero id faucibus nisl tincidunt eget nullam. Aliquam faucibus purus in massa tempor nec feugiat. Varius quam quisque id diam vel quam. Cras fermentum odio eu feugiat pretium nibh ipsum consequat nisl. Volutpat blandit aliquam etiam erat velit scelerisque in dictum. Purus faucibus ornare suspendisse sed nisi lacus sed viverra. Commodo sed egestas egestas fringilla phasellus faucibus. In ornare quam viverra orci sagittis eu volutpat odio facilisis. Cursus turpis massa tincidunt dui ut. Est lorem ipsum dolor sit amet. Feugiat nisl pretium fusce id velit ut tortor pretium viverra. Ullamcorper malesuada proin libero nunc consequat interdum varius. Quis blandit turpis cursus in hac habitasse. Donec adipiscing tristique risus nec feugiat in fermentum posuere. Platea dictumst quisque sagittis purus sit amet volutpat consequat mauris. Cras ornare arcu dui vivamus arcu felis. Vestibulum rhoncus est pellentesque elit ullamcorper dignissim cras tincidunt. Feugiat in fermentum posuere urna nec tincidunt. Ac odio tempor orci dapibus ultrices in iaculis nunc. Nibh cras pulvinar mattis nunc sed blandit libero volutpat sed. Nisi quis eleifend quam adipiscing vitae proin sagittis nisl. Nulla malesuada pellentesque elit eget gravida cum sociis. Vitae sapien pellentesque habitant morbi tristique senectus. Lacinia quis vel eros donec ac odio. Sollicitudin tempor id eu nisl nunc. Mauris cursus mattis molestie a iaculis at erat pellentesque. At quis risus sed vulputate odio ut. At imperdiet dui accumsan sit amet. Egestas dui id ornare arcu odio ut sem nulla pharetra. Dapibus ultrices in iaculis nunc. Sit amet justo donec enim diam vulputate. Ultrices tincidunt arcu non sodales neque sodales. Facilisis leo vel fringilla est ullamcorper eget. Nibh cras pulvinar mattis nunc sed blandit libero volutpat sed";
