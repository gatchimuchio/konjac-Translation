use std::net::SocketAddr;
use std::process::ExitCode;

mod http;
use konjac_translation::CONTRACT_VERSION;

const HELP: &str = "こんにゃく翻訳 — local text experience\n\nUsage:\n  konjac-translation serve [--bind 127.0.0.1:8787]\n  konjac-translation --help\n\nThe local UI and API accept typed text, preserve the selected target languages, and report the audited MINIDORA capability state. Requests remain in memory for one response. No translation is claimed while the selected engine capability is unavailable.\n\nAPI contract: ";

fn parse_bind(args: impl IntoIterator<Item = String>) -> Result<Option<SocketAddr>, String> {
    let mut args = args.into_iter();
    let first = args.next();
    match first.as_deref() {
        Some("--help") | Some("-h") | None => Ok(None),
        Some("serve") => {
            let mut bind: SocketAddr = "127.0.0.1:8787".parse().expect("literal address");
            while let Some(option) = args.next() {
                if option == "--bind" {
                    let value = args.next().ok_or("--bind requires an IP:port value")?;
                    bind = value
                        .parse()
                        .map_err(|_| "--bind requires a numeric IP:port value")?;
                } else {
                    return Err("unknown option; use --help".into());
                }
            }
            if !bind.ip().is_loopback() {
                return Err("--bind must use a loopback IP address".into());
            }
            Ok(Some(bind))
        }
        Some(_) => Err("unknown command; use --help".into()),
    }
}

fn main() -> ExitCode {
    match parse_bind(std::env::args().skip(1)) {
        Ok(None) => {
            println!("{HELP}{CONTRACT_VERSION}");
            ExitCode::SUCCESS
        }
        Ok(Some(bind)) => match http::serve(bind) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("konjac-translation: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("konjac-translation: {error}");
            ExitCode::FAILURE
        }
    }
}
