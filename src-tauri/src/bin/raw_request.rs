use omnichat_lib::services::provider::ProviderService;

#[tokio::main]
async fn main() {
    omnichat_lib::services::logging::init_logging();
    if let Err(err) = run().await {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let args = parse_args(std::env::args().skip(1).collect())?;

    let response = ProviderService::send_raw_request_direct(
        &args.base_url,
        args.api_key.as_deref(),
        &args.method,
        &args.path,
        args.headers,
        args.body,
    )
    .await
    .map_err(|err| err.to_string())?;

    println!("URL: {}", response.url);
    println!("STATUS: {}", response.status);
    println!("HEADERS:");
    for (name, value) in response.headers {
        println!("{name}: {value}");
    }
    println!();
    println!("BODY:");
    println!("{}", response.body);

    Ok(())
}

struct CliArgs {
    base_url: String,
    api_key: Option<String>,
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

fn parse_args(args: Vec<String>) -> Result<CliArgs, String> {
    let mut base_url = None;
    let mut api_key = None;
    let mut method = None;
    let mut path = None;
    let mut headers = Vec::new();
    let mut body = None;
    let mut body_file = None;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--base-url" => {
                i += 1;
                base_url = Some(value_at(&args, i, "--base-url")?.to_string());
            }
            "--api-key" => {
                i += 1;
                api_key = Some(value_at(&args, i, "--api-key")?.to_string());
            }
            "--method" => {
                i += 1;
                method = Some(value_at(&args, i, "--method")?.to_string());
            }
            "--path" => {
                i += 1;
                path = Some(value_at(&args, i, "--path")?.to_string());
            }
            "--header" => {
                i += 1;
                let raw = value_at(&args, i, "--header")?;
                let (name, value) = raw
                    .split_once(':')
                    .ok_or_else(|| format!("invalid header '{raw}', expected 'Name: Value'"))?;
                headers.push((name.trim().to_string(), value.trim().to_string()));
            }
            "--body" => {
                i += 1;
                body = Some(value_at(&args, i, "--body")?.to_string());
            }
            "--body-file" => {
                i += 1;
                body_file = Some(value_at(&args, i, "--body-file")?.to_string());
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => {
                return Err(format!("unknown argument: {other}"));
            }
        }
        i += 1;
    }

    if body.is_some() && body_file.is_some() {
        return Err("use either --body or --body-file, not both".to_string());
    }

    let body = match (body, body_file.as_deref()) {
        (Some(body), None) => Some(body),
        (None, Some(path)) => Some(
            std::fs::read_to_string(path)
                .map_err(|err| format!("failed to read body file '{path}': {err}"))?,
        ),
        (None, None) => None,
        (Some(_), Some(_)) => unreachable!(),
    };

    Ok(CliArgs {
        base_url: base_url.ok_or_else(|| "missing --base-url".to_string())?,
        api_key,
        method: method.unwrap_or_else(|| "GET".to_string()),
        path: path.ok_or_else(|| "missing --path".to_string())?,
        headers,
        body,
    })
}

fn value_at<'a>(args: &'a [String], index: usize, flag: &str) -> Result<&'a str, String> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("missing value for {flag}"))
}

fn print_help() {
    println!("Usage:");
    println!(
        "  cargo run --bin raw_request -- --base-url <url> --method <GET|POST> --path <path> [--api-key <key>] [--header \"Name: Value\"]... [--body <text> | --body-file <file>]"
    );
}
