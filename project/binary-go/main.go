package main

import (
    "crypto/aes"
    "encoding/hex"
    "fmt"
    "log"
    "math/rand"
    "net/http"
    "strings"

    "github.com/gorilla/websocket"
    "go.bug.st/serial/enumerator"
    "go.bug.st/serial"
)

type Handler func(http.ResponseWriter, *http.Request)


var PORT_MODE = serial.Mode{
    BaudRate: 9600,
    Parity:   serial.NoParity,
    DataBits: 8,
    StopBits: serial.OneStopBit,
}
const PORT_MODE_STR = "BaudRate: 9600, Parity: NoParity, DataBits: 8, StopBits: OneStopBit"

const (
    BLOCK_SIZE int = 16;
    KEY_SIZE int = 16;
)

var upgrader = websocket.Upgrader{
    ReadBufferSize:  1024,
    WriteBufferSize: 1024,
}

type Logger struct {
    msgs chan string
    conn *websocket.Conn
}

func (l *Logger) Init() *Logger {
    l.msgs = make(chan string, 100)
    l.conn = nil
    log.SetOutput(l)
    log.Default().SetFlags(log.Ltime)
    return l
}

func (l Logger) Write(p []byte) (n int, err error) {
    l.msgs <- string(p)
    fmt.Print(string(p))
    return len(p), nil
}

func (l* Logger) SetConn(conn *websocket.Conn) {
    l.conn = conn
}

func (l *Logger) handle_ws(w http.ResponseWriter, r *http.Request) {
    conn, err := upgrader.Upgrade(w, r, nil)
    if err != nil {
        fmt.Fprintf(w, "%+v\n", err)
    }
    l.SetConn(conn)
    go l.doLogging()
}

func (l *Logger) fmtMsg(msg string) string {
    return fmt.Sprintf(`
        <div id="log" hx-swap-oob="beforeend">
            <p class="font-mono">%s</p>
        </div>
    `, msg)
}

func (l *Logger) doLogging() {
    defer func() {
        l.conn.Close()
        l.conn = nil
    }()
    for {
        rawMsg, stillOpen := <- l.msgs
        if !stillOpen {
            return
        }
        msg := l.fmtMsg(rawMsg)
        err := l.conn.WriteMessage(websocket.TextMessage, []byte(msg))
        if err != nil {
            fmt.Println(err)
            return
        }
    }
}

func main() {
    sha := new(BAESys128)
    logger := new(Logger).Init()

    ports, err := enumerator.GetDetailedPortsList()
    if err != nil {
        log.Fatal(err)
    }
    log.Println(ports)

    found := false;
    for _, port := range ports {
        if port.VID == "0403" && port.PID == "6010" {
            log.Println("Found Basys3 at", port.Name)
            port, err := serial.Open(port.Name, &serial.Mode{})
            if err != nil {
                log.Fatalf("Error opening port: %s", err)
            }
            port.SetMode(&PORT_MODE)
            log.Printf("Opened port with mode <code>%s</code>", PORT_MODE_STR)
            sha.SetPort(&port)
            found = true;
            break;
        }
    }
    if !found {
        log.Printf("Could not find Basys3")
    }
    http.HandleFunc("/", index)
    http.HandleFunc("/submit", handle_submit)
    http.HandleFunc("/key", handle_set_key(sha))
    http.HandleFunc("/encrypt", handle_encrypt_message(sha))
    http.HandleFunc("/key/random", handle_random_key)
    http.HandleFunc("/message/random", handle_random_message)
    http.HandleFunc("/ws", logger.handle_ws)

    // Start the server on port 8080
    fmt.Println("Server running on http://localhost:8080")
    log.Println("Server started at <code>http://localhost:8080</code>")
    err = http.ListenAndServe(":8080", nil)
    close(logger.msgs)
    if err != nil {
        fmt.Printf("Error starting server: %s\n", err)
    }
}

type PageFormOpts struct {
    key *string;
    key_err *string;
    message *string;
    ciphertext *string;
    encrypt_err *string;
}

func index(w http.ResponseWriter, r *http.Request) {
    var opts PageFormOpts
    fmt.Fprintf(w, `
    <html>
        <head>
            <title>Basys3 AES Server</title>
            <meta name="viewport" content="width=device-width, initial-scale=1" />
            <meta charset="utf-8" />
            <script src="https://unpkg.com/htmx.org@1.9.6"></script>
            <script src="https://unpkg.com/htmx.org/dist/ext/ws.js"></script>
            <script src="https://cdn.tailwindcss.com"></script>
            <style>
                code {
                    background: #3465a424;
                    border-radius: 2px;
                }
            </style>
        </head>
        <body class="px-10 py-10">
            <div class="flex flex-row justify-between">
                %s
                <div>
                    <label for="log">System Log</label>
                    <div hx-ext="ws" ws-connect="/ws" id="log" class="w-[600px] h-[400px] overflow-auto border-2"></div>
                </div>
            </div>
        <h2>Log</h2>
        </body>
    </html>
        `, opts.render())
}

func (opts PageFormOpts) render() string {
    return fmt.Sprintf(`
            <form hx-post="/submit" hx-swap="outerHTML" id="form">
                %s
                %s
                %s
            </form>
        `,
        key_form_group(opts.key, opts.key_err),
        message_form_group(opts.message),
        cipher_form_group(opts.ciphertext, opts.encrypt_err),
    )
}

func parse_form(r *http.Request) PageFormOpts {
    var opts PageFormOpts
    key := r.FormValue("key")
    opts.key = &key
    message := r.FormValue("message")
    opts.message = &message
    return opts
}


func key_form_group(key *string, key_err *string) string {
    return fmt.Sprintf(`
            <div id="key-part" class="flex flex-row gap-2">
                <label for="key">Secret Key</label>
                %s
                <button hx-post="/key" hx-target="#form" class="border-2 bg-slate-100">
                    Set
                </button>
                <button class="border-2 bg-slate-100" hx-get="/key/random" hx-target="#key-input">
                    Random Key
                </button>
            </div>
            %s
        `,
        key_input(key),
        error_p("key-error", key_err, false),
    )
}

func error_p(id string, err *string, out_of_band bool) string {
    label := ""
    if err != nil {
        label = "ERROR: "
    }
    oob := ""
    if out_of_band {
        oob = `hx-swap-oob="true"`
    }
    error := empty_if_nil(err)
    return fmt.Sprintf(`
        <p id="%s" %s style="color: #FF0000; font-size: 14px; font-weight: bold; margin-top: 5px;">%s%s</p>
    `, id, oob, label, error)
}

func key_input(_key *string) string {
    key := empty_if_nil(_key)
    return fmt.Sprintf(`
        <input spellcheck="false" type="text" id="key-input" class="border-2" name="key" value="%s"></input>
    `, key)
}

func message_form_group(_message *string) string {
    message := empty_if_nil(_message)
    return fmt.Sprintf(`
            <div id="message-part" class="flex flex-col gap-2 py-4">
                <label for="message">Plain Text:</label>
                <textarea spellcheck="false" type="text" id="message" name="message" class="w-[600px] border-2" rows="4" >%s</textarea>
                <div class="flex flex-row justify-start gap-2">
                   <button class="border-2 bg-slate-100" hx-get="/message/random" hx-target="#message" hx-swap="innerHTML">
                       Random Message
                   </button>
                   <button hx-post="/encrypt" hx-target="form" class="border-2 bg-slate-100">
                      Encrypt!
                   </button>
                </div>
            </div>
        `, message)
}

func cipher_form_group(_ct *string, err *string) string {
    ct := empty_if_nil(_ct)
    return fmt.Sprintf(`
            <div id="cipher-part" class="flex flex-col gap-2 py-2">
                <p>Cipher Text</p>
                <textarea readonly class="w-[600px] h-[200px] border-2 break-words">%s</textarea>
                <button hx-post="/encrypt" hx-target="form" class="block border-2 bg-slate-100">
                   Decrypt (TODO!)
                </button>
                %s
            </div>
        `, ct, error_p("encrypt-error", err, false))
}

func empty_if_nil(s *string) string {
    if s == nil {
        return ""
    }
    return *s
}

func handle_submit(w http.ResponseWriter, r *http.Request) {
    opts := parse_form(r)
    fmt.Fprint(w, opts.render())
}

func handle_set_key(sha *BAESys128) Handler {
    return func(w http.ResponseWriter, r *http.Request) {
        opts := parse_form(r)
        opts.key_err = validate_key(opts.key)
        log.Printf("Set key to <code>%s</code>. Error: <code>%s</code>", empty_if_nil(opts.key), empty_if_nil(opts.key_err))
        if opts.key_err == nil {
            sha.SetKey([]byte(*opts.key))
        }
        fmt.Fprint(w, opts.render())
    }
}

func handle_random_key(w http.ResponseWriter, r *http.Request) {
    fmt.Fprint(w, key_input(gen_random_key()))
    fmt.Fprint(w, error_p("key-error", nil, true))
}

func gen_random_key() *string {
    charset := []byte("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$^&*-=+?")
    key := make([]byte, 16)
    for i := range key {
        key[i] = charset[rand.Intn(len(charset))]
    }
    var keystr = string(key)
    log.Printf("Generated key <code>%s</code>", keystr)
    return &keystr
}

func handle_random_message(w http.ResponseWriter, r *http.Request) {
    fmt.Fprint(w, gen_random_message())
}

func gen_random_message() string {
    words := strings.Split(LOREM_IPSUM, " ")
    rand.Shuffle(len(words), func(i, j int) {
        words[i], words[j] = words[j], words[i]
    })
    num_words := rand.Intn(len(words))
    msg := strings.Join(words[:num_words], " ")
    log.Printf("Generated message of length <code>%d</code>", len(msg))
    return msg
}

func validate_key(key *string) *string {
    var msg = ""
    if key == nil {
        msg = "Key is required"
    }
    if len(*key) != 16 {
        msg = fmt.Sprintf("Key is %d characters long but it must be 16 characters long", len(*key))
    }
    if msg == "" {
        return nil
    }
    return &msg
}

type BAESys128 struct {
    key []byte;
    buf []byte;
    port *serial.Port;
}

func (s * BAESys128) SetPort(port *serial.Port) {
    s.port = port;
}

func (s *BAESys128) Write(p []byte) (n int, err error) {
    cipher, err := aes.NewCipher(s.key)
    if err != nil {
        return 0, err
    }
    dest := make([]byte,BLOCK_SIZE)
    cipher.Encrypt(dest, p)
    s.buf = dest
    if s.port != nil {
        _, err = (*s.port).Write(p)
        if err != nil {
            log.Printf("Failed to write to Basys3: <code>%s</code>", err)
        }
    }
    return len(p), nil
}

func (s *BAESys128) Read() []byte {
    res := s.buf
    s.buf = nil
    if s.port == nil {
        return res
    }
    portRes := make([]byte, BLOCK_SIZE)
    _, err := (*s.port).Read(portRes)
    if err != nil {
        log.Printf("Failed to read from Basys3: <code>%s</code>", err)
        return res
    }
    if len(portRes) != BLOCK_SIZE {
        log.Printf("Read <code>%d</code> bytes from Basys3 but expected <code>%d</code>", len(portRes), BLOCK_SIZE)
        return res
    }
    if string(portRes) != string(res) {
        log.Printf("Read <code>%s</code> from Basys3 but expected <code>%s</code>", hex.EncodeToString(portRes), hex.EncodeToString(res))
    }
    log.Println("Returning bytes read from Basys3")
    return portRes
}

func pkcs7Pad(data []byte) []byte {
    padding := BLOCK_SIZE - (len(data) % BLOCK_SIZE)
    if padding == BLOCK_SIZE || padding == 0 {
        return data
    }
    padBytes := make([]byte, padding)
    log.Printf("Adding pad of length <code>%d</code", padding)
    for i := range padBytes {
        padBytes[i] = byte(padding)
    }

    return append(data, padBytes...)
}

func (s *BAESys128) Blocks(msg []byte) [][]byte {
    msg = pkcs7Pad(msg)
    var blocks [][]byte
    for i := 0; i < len(msg); i += 16 {
        blocks = append(blocks, msg[i:i+16])
    }
    log.Printf("Split message into <code>%d</code> blocks", len(blocks))
    return blocks
}

func (s *BAESys128) Encrypt(msg []byte) ([]byte, error) {
    blocks := s.Blocks(msg)
    var ct []byte
    if s.port == nil {
        log.Println("No port set. Encrypting without Basys3")
    }
    for _, block := range blocks {
        _, err := s.Write(block)
        if err != nil {
            return nil, err
        }
        ct = append(ct, s.Read()...)
    }
    return ct, nil
}

func (s *BAESys128) SetKey(key []byte) error {
    s.key = key;
    return nil
}

type EncryptResult struct {
    ciphertext *string;
    err error;
}

func handle_encrypt_message(sha *BAESys128) Handler {
    return func(w http.ResponseWriter, r *http.Request) {
        opts := parse_form(r)
        hasKey := len(sha.key) != 0
        sentKey := opts.key != nil && len(*opts.key) != 0
        sameKey := hasKey && sentKey && string(sha.key) == *opts.key

        if ((!hasKey && sentKey) || !sameKey) {
            sha.SetKey([]byte(*opts.key))
        }
        key := string(sha.key)
        opts.key_err = validate_key(&key)
        if opts.key_err != nil || opts.key == nil {
            log.Println("Found Key error while trying to encrypt:", opts.key_err)
            fmt.Fprint(w, opts.render())
            return
        }
        log.Printf("Encrypting message of length <code>%d</code>", len(*opts.message))
        ct, err := sha.Encrypt([]byte(*opts.message))
        if err != nil {
            err_msg := err.Error()
            log.Printf("Error while trying to encrypt: <code>%s</code>", err_msg)
            opts.encrypt_err = &err_msg
        }
        opts.ciphertext = new(string)
        log.Printf("Encrypted message to ciphertext of length <code>%d</code>", len(ct))
        *opts.ciphertext = strings.ToUpper(hex.EncodeToString(ct))
        fmt.Fprint(w, opts.render())
    }
}

const LOREM_IPSUM string = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. A erat nam at lectus urna duis convallis convallis. Ut placerat orci nulla pellentesque dignissim enim sit amet venenatis. Interdum consectetur libero id faucibus nisl tincidunt eget nullam non. Diam vulputate ut pharetra sit amet aliquam id. Nisi lacus sed viverra tellus in hac habitasse platea dictumst. Mi proin sed libero enim sed faucibus turpis in eu. Vulputate odio ut enim blandit volutpat maecenas volutpat. Nulla facilisi nullam vehicula ipsum a. Tellus in hac habitasse platea dictumst vestibulum rhoncus est pellentesque. In tellus integer feugiat scelerisque varius morbi enim nunc faucibus. Auctor urna nunc id cursus metus aliquam eleifend mi. In hac habitasse platea dictumst vestibulum. Adipiscing bibendum est ultricies integer quis auctor. Risus quis varius quam quisque id. Nisl condimentum id venenatis a. Vitae sapien pellentesque habitant morbi tristique senectus et netus et. Ultrices gravida dictum fusce ut placerat orci nulla pellentesque dignissim. Ullamcorper morbi tincidunt ornare massa eget egestas purus. Amet massa vitae tortor condimentum. Tristique et egestas quis ipsum. Pulvinar mattis nunc sed blandit libero volutpat. Interdum velit euismod in pellentesque massa placerat. Tellus elementum sagittis vitae et leo duis ut diam. Nisl rhoncus mattis rhoncus urna neque viverra justo nec. Arcu ac tortor dignissim convallis aenean et tortor. Faucibus interdum posuere lorem ipsum dolor. At ultrices mi tempus imperdiet. Velit sed ullamcorper morbi tincidunt. Sed viverra ipsum nunc aliquet bibendum enim facilisis gravida. Mauris pellentesque pulvinar pellentesque habitant morbi tristique senectus et. Nibh sit amet commodo nulla facilisi nullam vehicula ipsum. Tristique nulla aliquet enim tortor at auctor urna nunc. Massa vitae tortor condimentum lacinia quis vel eros donec. Lorem sed risus ultricies tristique nulla aliquet enim. Maecenas volutpat blandit aliquam etiam erat velit scelerisque in dictum. Cras tincidunt lobortis feugiat vivamus at augue eget arcu. Ultricies mi eget mauris pharetra et ultrices neque ornare. Diam quis enim lobortis scelerisque fermentum dui faucibus in ornare. Molestie ac feugiat sed lectus vestibulum mattis. Enim sed faucibus turpis in. Lectus urna duis convallis convallis tellus id. Cursus risus at ultrices mi tempus imperdiet nulla malesuada. Libero id faucibus nisl tincidunt eget nullam. Aliquam faucibus purus in massa tempor nec feugiat. Varius quam quisque id diam vel quam. Cras fermentum odio eu feugiat pretium nibh ipsum consequat nisl. Volutpat blandit aliquam etiam erat velit scelerisque in dictum. Purus faucibus ornare suspendisse sed nisi lacus sed viverra. Commodo sed egestas egestas fringilla phasellus faucibus. In ornare quam viverra orci sagittis eu volutpat odio facilisis. Cursus turpis massa tincidunt dui ut. Est lorem ipsum dolor sit amet. Feugiat nisl pretium fusce id velit ut tortor pretium viverra. Ullamcorper malesuada proin libero nunc consequat interdum varius. Quis blandit turpis cursus in hac habitasse. Donec adipiscing tristique risus nec feugiat in fermentum posuere. Platea dictumst quisque sagittis purus sit amet volutpat consequat mauris. Cras ornare arcu dui vivamus arcu felis. Vestibulum rhoncus est pellentesque elit ullamcorper dignissim cras tincidunt. Feugiat in fermentum posuere urna nec tincidunt. Ac odio tempor orci dapibus ultrices in iaculis nunc. Nibh cras pulvinar mattis nunc sed blandit libero volutpat sed. Nisi quis eleifend quam adipiscing vitae proin sagittis nisl. Nulla malesuada pellentesque elit eget gravida cum sociis. Vitae sapien pellentesque habitant morbi tristique senectus. Lacinia quis vel eros donec ac odio. Sollicitudin tempor id eu nisl nunc. Mauris cursus mattis molestie a iaculis at erat pellentesque. At quis risus sed vulputate odio ut. At imperdiet dui accumsan sit amet. Egestas dui id ornare arcu odio ut sem nulla pharetra. Dapibus ultrices in iaculis nunc. Sit amet justo donec enim diam vulputate. Ultrices tincidunt arcu non sodales neque sodales. Facilisis leo vel fringilla est ullamcorper eget. Nibh cras pulvinar mattis nunc sed blandit libero volutpat sed"
