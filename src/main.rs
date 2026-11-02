use std::time::Duration;

use anyhow::{Context, Result};
use image::RgbaImage;

use rgrim::capture::{capture_primary_monitor, capture_settled_monitor};
use rgrim::editor::{EditorOutcome, ToolPalette, crop_image, run_editor};
use rgrim::export::{generate_screenshot_filename, get_screenshot_directory};
use rgrim::ui::run_sniper_overlay;

const USAGE: &str = "\
rgrim - lightweight screenshot sniper

Usage: rgrim [OPTIONS]

Options:
  -g, --gui              Open the dashboard instead of capturing immediately
  -d, --delay <SECONDS>  Wait SECONDS before the first capture (e.g. --delay 3)
  -h, --help             Print this help
  -V, --version          Print version";

/// Parsed command-line options.
struct Cli {
    gui: bool,
    delay: Duration,
}

/// Result of parsing the command line.
enum Parsed {
    Run(Cli),
    Help,
    Version,
}

fn main() -> Result<()> {
    let cli = match parse_args(std::env::args().skip(1))? {
        Parsed::Run(cli) => cli,
        Parsed::Help => {
            println!("{USAGE}");
            return Ok(());
        }
        Parsed::Version => {
            println!("rgrim {}", rgrim::VERSION);
            return Ok(());
        }
    };

    if cli.gui {
        if let Some(delay) = rgrim::dashboard::run()? {
            trigger_instant_capture_flow(delay)?;
        }
    } else {
        trigger_instant_capture_flow(cli.delay)?;
    }

    Ok(())
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Parsed> {
    let mut cli = Cli {
        gui: false,
        delay: Duration::ZERO,
    };
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--gui" | "-g" => cli.gui = true,
            "--help" | "-h" => return Ok(Parsed::Help),
            "--version" | "-V" => return Ok(Parsed::Version),
            "--delay" | "-d" => {
                let value = args.next().context("--delay expects a number of seconds")?;
                cli.delay = parse_seconds(&value)?;
            }
            other => {
                let value = other
                    .strip_prefix("--delay=")
                    .with_context(|| format!("unrecognized argument `{other}`\n\n{USAGE}"))?;
                cli.delay = parse_seconds(value)?;
            }
        }
    }

    Ok(Parsed::Run(cli))
}

fn parse_seconds(value: &str) -> Result<Duration> {
    let seconds = value
        .parse::<u64>()
        .with_context(|| format!("`{value}` is not a whole number of seconds"))?;
    Ok(Duration::from_secs(seconds))
}

/// Announces and waits for `delay` before the next capture, printing one
/// line per remaining second so a hotkey-triggered run is not silent.
fn wait_before_capture(delay: Duration) {
    for remaining in (1..=delay.as_secs()).rev() {
        println!("Capturing in {remaining}…");
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn trigger_instant_capture_flow(delay: Duration) -> Result<()> {
    wait_before_capture(delay);
    let mut captured = capture_primary_monitor()?;
    let mut palette = ToolPalette::default();

    loop {
        let rect = run_sniper_overlay(&captured.image)?;

        let Some(region) = rect else {
            println!("Selection cancelled.");
            return Ok(());
        };

        let cropped = crop_image(&captured.image, &region);

        let auto_save_msg = try_auto_save(&cropped);

        let (outcome, updated_palette) = run_editor(cropped, auto_save_msg, palette)?;
        palette = updated_palette;

        match outcome {
            EditorOutcome::NewCapture { delay } => {
                wait_before_capture(delay);
                // The compositor may still be rendering the closed editor
                // window; capture_settled_monitor() waits for stable output
                // instead of relying on a guessed delay.
                captured = capture_settled_monitor()?;
            }
            EditorOutcome::Closed => return Ok(()),
        }
    }
}

/// Best-effort auto-save of the cropped screenshot. Logs and returns
/// `None` on any failure, since the editor can still save manually.
fn try_auto_save(cropped: &RgbaImage) -> Option<String> {
    let save_dir = get_screenshot_directory();
    std::fs::create_dir_all(save_dir)
        .inspect_err(|e| eprintln!("Failed to create screenshot directory: {e}"))
        .ok()?;

    let full_path = save_dir.join(generate_screenshot_filename());
    cropped
        .save(&full_path)
        .inspect_err(|e| eprintln!("Auto-save failed: {e}"))
        .ok()?;

    Some(format!("Screenshot saved to {}", full_path.display()))
}

#[cfg(test)]
mod tests {
    use super::{Cli, Parsed, parse_args};
    use std::time::Duration;

    fn run(args: &[&str]) -> Cli {
        match parse_args(args.iter().map(|arg| arg.to_string())).expect("valid arguments") {
            Parsed::Run(cli) => cli,
            Parsed::Help => panic!("expected Run, got Help"),
            Parsed::Version => panic!("expected Run, got Version"),
        }
    }

    #[test]
    fn defaults_to_an_immediate_capture() {
        let cli = run(&[]);

        assert!(!cli.gui);
        assert_eq!(cli.delay, Duration::ZERO);
    }

    #[test]
    fn parses_gui_flag() {
        assert!(run(&["--gui"]).gui);
        assert!(run(&["-g"]).gui);
    }

    #[test]
    fn parses_delay_in_all_spellings() {
        assert_eq!(run(&["--delay", "3"]).delay, Duration::from_secs(3));
        assert_eq!(run(&["--delay=5"]).delay, Duration::from_secs(5));
        assert_eq!(run(&["-d", "10"]).delay, Duration::from_secs(10));
    }

    #[test]
    fn rejects_missing_delay_value() {
        assert!(parse_args(["--delay".to_string()]).is_err());
    }

    #[test]
    fn rejects_non_numeric_delay() {
        assert!(parse_args(["--delay".to_string(), "soon".to_string()]).is_err());
    }

    #[test]
    fn rejects_unknown_arguments() {
        assert!(parse_args(["--nope".to_string()]).is_err());
    }

    #[test]
    fn help_flag_short_circuits() {
        let parsed = parse_args(["--help".to_string()]).expect("help parses");

        assert!(matches!(parsed, Parsed::Help));
    }

    #[test]
    fn version_flags_short_circuit() {
        for flag in ["-V", "--version"] {
            let parsed = parse_args([flag.to_string()]).expect("version parses");

            assert!(matches!(parsed, Parsed::Version), "{flag}");
        }
    }
}
