use std::rc::Rc;
use std::sync::Arc;

// use aes::Aes128;
use aes::cipher::{generic_array::GenericArray, BlockEncrypt, KeyInit};
use aes::Aes128;
use anyhow::{anyhow, Context, Result};
use axum::extract::State;
use axum::{
    http::HeaderMap,
    response::Html,
    routing::{get, post},
    Form, Router,
};
use formatx::*;
use len_trait::Len;
use rand::Rng;
use tokio::sync::Mutex;

type Partial = Html<&'static str>;

const KEY_LEN: usize = 16;
const BLOCK_SIZE: usize = 16;

// fn get_right_port(ports: Vec<serialport::SerialPortInfo>) -> Option<Vec<serialport::SerialPortInfo>> {
//     return ports.into_iter().filter(
//         |port| {
//             match (port.port_type) {
//                 serialport::UsbPort(info) => match (info.vid, info.manufacturer) {
//                     (1027, Some("Future Technology Devices International, Ltd")) => true,
//                     _ => Some,
//                     },
//                 _ => false
//             }
//         }
//     );
// }

#[derive(Clone)]
struct Uart {
    port: Arc<Mutex<Box<dyn serialport::SerialPort>>>,
    key: Option<[u8; KEY_LEN]>,
}

impl Uart {
    fn init(port_name: String) -> Result<Self> {
        let mut p = serialport::new(&port_name, 9600)
            .open()
            .context("can open port")?;
        return Ok(Self {
            port: Arc::new(Mutex::new(p)),
            key: None,
        });
    }

    async fn set_key(&mut self, key: Option<&str>) -> Result<()> {
        let key = match key {
            Some(key) => key,
            None => return Ok(())
        };
        let mut key_arr = [0; KEY_LEN];
        for (i, b) in key.bytes().enumerate() {
            key_arr[i] = b;
        }
        self.key = Some(key_arr);
        // FIXME: how will basys signal success?
        let mut port = self.port.lock().await;
        port.write_all(&key_arr)?;
        Ok(())
    }

    fn encrypt_no_uart(&self, pt: &str) -> Result<String> {
        type Block = GenericArray<u8, aes::cipher::typenum::U16>;

        let key = match self.key {
            Some(key) => GenericArray::from(key),
            None => return Err(anyhow!("Key not set")),
        };

        let num_blocks = (pt.len() + KEY_LEN) / KEY_LEN;
        let mut blocks = vec![Block::default(); num_blocks];
        let pt_bytes = pt.as_bytes();

        for i in (0..pt.len()).step_by(KEY_LEN) {
            let bytes = &pt_bytes[i..usize::min(i + KEY_LEN, pt.len() - 1)];
            let block = &mut blocks[i / KEY_LEN];

            for b in 0..bytes.len() {
                // dbg!("{} {} {}", b, block.len(), bytes.len());
                block[b] = bytes[b]
            }
        }

        let cipher = Aes128::new(&key);
        cipher.encrypt_blocks(&mut blocks);

        let ct: Vec<u8> = blocks.into_iter().flatten().collect();

        Ok(hex::encode_upper(ct))
    }

    async fn encrypt_uart(&mut self, pt: &str) -> Result<String> {
        type Block = [u8; BLOCK_SIZE];

        let num_blocks = (pt.len() + BLOCK_SIZE) / BLOCK_SIZE;
        let mut blocks = vec![Block::default(); num_blocks];
        let pt_bytes = pt.as_bytes();

        for i in (0..pt.len()).step_by(BLOCK_SIZE) {
            let bytes = &pt_bytes[i..usize::min(i + BLOCK_SIZE, pt.len() - 1)];
            let block = &mut blocks[i / BLOCK_SIZE];

            for b in 0..bytes.len() {
                // dbg!("{} {} {}", b, block.len(), bytes.len());
                block[b] = bytes[b]
            }
        }

        let mut ct = vec![[0; BLOCK_SIZE]; num_blocks];
        let mut port = self.port.lock().await;

        for (i, block) in blocks.into_iter().enumerate() {
            let written = port.write(&block)?;
            anyhow::ensure!(
                written == BLOCK_SIZE,
                "Wrote BLOCK_SIZE ({}) bytes",
                written
            );
            port.read_exact(&mut ct[i])?;
        }

        let ct = hex::encode_upper(ct.into_iter().flatten().collect::<Vec<u8>>());

        Ok(ct)
    }

    async fn encrypt(&mut self, pt: &str) -> Result<String> {
        let uart_res = self.encrypt_uart(pt).await;
        if let Ok(ct) = uart_res {
            return Ok(ct);
        }
        eprintln!("Failed to encrypt with uart: {:?}", uart_res);
        return self.encrypt_no_uart(pt);
    }
}

#[tokio::main]
async fn main() {
    let ports = serialport::available_ports().expect("can look for ports");
    let mut found = vec![];
    for port in ports.iter() {
        match &port.port_type {
            serialport::SerialPortType::UsbPort(info) => match info.vid {
                1027 => {
                    found.push(port.to_owned());
                }
                _ => {}
            },
            // PciPort ??
            _ => {}
        }
    }
    dbg!("{:?}", &found);

    if found.len() == 0 {
        panic!("No basys3 found");
    }

    let uart = Uart::init(found[0].port_name.clone()).expect("can init uart");

    let app = Router::new()
        .route("/", get(render(index())))
        .route("/submit", post(handle_submit))
        .route("/key", post(handle_set_key))
        .route("/encrypt", post(handle_encrypt_message))
        .route("/key/random", get(handle_random_key))
        .route("/message/random", get(handle_random_message))
        .with_state(uart);

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
    pub encrypt_err: Option<String>,
}

impl PageFormOpts {
    fn clean(self) -> Self {
        Self {
            key: none_if_empty(self.key),
            key_err: none_if_empty(self.key_err),
            message: none_if_empty(self.message),
            ciphertext: none_if_empty(self.ciphertext),
            encrypt_err: none_if_empty(self.encrypt_err),
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
            encrypt_err: None,
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

    return render(format!("{}{}", key_input(Some(key)), error("key-error",None, true)));
}

async fn handle_random_message() -> Partial {
    let text = gen_random_message();
    return Html(text.leak());
}

async fn handle_set_key(State(mut state): State<Uart>, Form(mut opts): Form<PageFormOpts>) -> Partial {
    let (key, key_err) = match validate_key(opts.key.clone()) {
        Ok(key) => (Some(key), None),
        Err(err) => (opts.key.clone(), Some(err.to_string())),
    };
    opts.key = key;
    opts.key_err = key_err;
    // FIXME: don't ignore
    let _ = state.set_key(opts.key.as_deref()).await;
    dbg!("{:?}", &opts.key);

    render(page_form(opts))
}

async fn handle_encrypt_message(State(mut state): State<Uart>, Form(opts): Form<PageFormOpts>) -> Partial {
    let mut opts = opts.clean();

    match &state.key {
        Some(_) => {},
        None => {
            let _ = state.set_key(opts.key.as_deref()).await;
        }
    }

    if let Some(message) = &opts.message {
        let ct = state.encrypt(&message).await;
        let (ct, err) = match ct {
            Ok(ct) => (Some(ct), None),
            Err(err) => (None, Some(err.to_string())),
        };
        opts.ciphertext = ct;
        opts.encrypt_err = err;
    }

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
        encrypt_err
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
        cipher_form_group(ciphertext, encrypt_err)
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
        error("key-error", key_err, false)
    )
}

fn error(id: &str, err: Option<String>, out_of_band: bool) -> String {
    let label = match err {
        Some(_) => "ERROR: ",
        None => "",
    };
    let err = err.unwrap_or_default();
    let oob = if out_of_band {
        r#"hx-swap-oob="true""#
    } else {
        ""
    };
    format!(
        r#"
            <p id="{}" {} style="color: #FF0000; font-size: 14px; font-weight: bold; margin-top: 5px;">{}{}</p>
    "#,
        id, oob, label, err
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

fn cipher_form_group(ct: Option<String>, err: Option<String>) -> TemplateResult {
    let ct = ct.unwrap_or_default();
    format!(
        r##"
            <div id="cipher-part" class="flex flex-col gap-2 py-2">
                <p>Cipher Text</p>
                <textarea readonly class="w-[600px] h-[200px] border-2 break-words">{}</textarea>
                <button hx-post="/encrypt" hx-target="form" class="block border-2 bg-slate-100">
                   Decrypt (TODO!)
                </button>
                {}
            </div>
        "##,
        ct,
        error("encrypt-error", err, false)
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

fn none_if_empty<S>(s: Option<S>) -> Option<S>
where
    S: Len,
{
    s.and_then(|s| if s.len() == 0 { None } else { Some(s) })
}

const LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. A erat nam at lectus urna duis convallis convallis. Ut placerat orci nulla pellentesque dignissim enim sit amet venenatis. Interdum consectetur libero id faucibus nisl tincidunt eget nullam non. Diam vulputate ut pharetra sit amet aliquam id. Nisi lacus sed viverra tellus in hac habitasse platea dictumst. Mi proin sed libero enim sed faucibus turpis in eu. Vulputate odio ut enim blandit volutpat maecenas volutpat. Nulla facilisi nullam vehicula ipsum a. Tellus in hac habitasse platea dictumst vestibulum rhoncus est pellentesque. In tellus integer feugiat scelerisque varius morbi enim nunc faucibus. Auctor urna nunc id cursus metus aliquam eleifend mi. In hac habitasse platea dictumst vestibulum. Adipiscing bibendum est ultricies integer quis auctor. Risus quis varius quam quisque id. Nisl condimentum id venenatis a. Vitae sapien pellentesque habitant morbi tristique senectus et netus et. Ultrices gravida dictum fusce ut placerat orci nulla pellentesque dignissim. Ullamcorper morbi tincidunt ornare massa eget egestas purus. Amet massa vitae tortor condimentum. Tristique et egestas quis ipsum. Pulvinar mattis nunc sed blandit libero volutpat. Interdum velit euismod in pellentesque massa placerat. Tellus elementum sagittis vitae et leo duis ut diam. Nisl rhoncus mattis rhoncus urna neque viverra justo nec. Arcu ac tortor dignissim convallis aenean et tortor. Faucibus interdum posuere lorem ipsum dolor. At ultrices mi tempus imperdiet. Velit sed ullamcorper morbi tincidunt. Sed viverra ipsum nunc aliquet bibendum enim facilisis gravida. Mauris pellentesque pulvinar pellentesque habitant morbi tristique senectus et. Nibh sit amet commodo nulla facilisi nullam vehicula ipsum. Tristique nulla aliquet enim tortor at auctor urna nunc. Massa vitae tortor condimentum lacinia quis vel eros donec. Lorem sed risus ultricies tristique nulla aliquet enim. Maecenas volutpat blandit aliquam etiam erat velit scelerisque in dictum. Cras tincidunt lobortis feugiat vivamus at augue eget arcu. Ultricies mi eget mauris pharetra et ultrices neque ornare. Diam quis enim lobortis scelerisque fermentum dui faucibus in ornare. Molestie ac feugiat sed lectus vestibulum mattis. Enim sed faucibus turpis in. Lectus urna duis convallis convallis tellus id. Cursus risus at ultrices mi tempus imperdiet nulla malesuada. Libero id faucibus nisl tincidunt eget nullam. Aliquam faucibus purus in massa tempor nec feugiat. Varius quam quisque id diam vel quam. Cras fermentum odio eu feugiat pretium nibh ipsum consequat nisl. Volutpat blandit aliquam etiam erat velit scelerisque in dictum. Purus faucibus ornare suspendisse sed nisi lacus sed viverra. Commodo sed egestas egestas fringilla phasellus faucibus. In ornare quam viverra orci sagittis eu volutpat odio facilisis. Cursus turpis massa tincidunt dui ut. Est lorem ipsum dolor sit amet. Feugiat nisl pretium fusce id velit ut tortor pretium viverra. Ullamcorper malesuada proin libero nunc consequat interdum varius. Quis blandit turpis cursus in hac habitasse. Donec adipiscing tristique risus nec feugiat in fermentum posuere. Platea dictumst quisque sagittis purus sit amet volutpat consequat mauris. Cras ornare arcu dui vivamus arcu felis. Vestibulum rhoncus est pellentesque elit ullamcorper dignissim cras tincidunt. Feugiat in fermentum posuere urna nec tincidunt. Ac odio tempor orci dapibus ultrices in iaculis nunc. Nibh cras pulvinar mattis nunc sed blandit libero volutpat sed. Nisi quis eleifend quam adipiscing vitae proin sagittis nisl. Nulla malesuada pellentesque elit eget gravida cum sociis. Vitae sapien pellentesque habitant morbi tristique senectus. Lacinia quis vel eros donec ac odio. Sollicitudin tempor id eu nisl nunc. Mauris cursus mattis molestie a iaculis at erat pellentesque. At quis risus sed vulputate odio ut. At imperdiet dui accumsan sit amet. Egestas dui id ornare arcu odio ut sem nulla pharetra. Dapibus ultrices in iaculis nunc. Sit amet justo donec enim diam vulputate. Ultrices tincidunt arcu non sodales neque sodales. Facilisis leo vel fringilla est ullamcorper eget. Nibh cras pulvinar mattis nunc sed blandit libero volutpat sed";
