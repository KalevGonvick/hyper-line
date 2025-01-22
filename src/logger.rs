use std::thread;
use env_logger::fmt::style::{Ansi256Color, Color, Style};
use std::io::Write;

pub fn setup_logger() {
    let level_filter = env_logger::Env::default().default_filter_or("DEBUG");
    env_logger::builder().parse_env(level_filter).format(|buf, record| {
        let level_colour: Style = match record.level() {
            log::Level::Error => {
                HighlightStyle::ErrorHighlight.style()
            }
            log::Level::Warn => {
                HighlightStyle::WarnHighLight.style()
            }
            log::Level::Info => {
                HighlightStyle::InfoHighlight.style()
            }
            log::Level::Debug => {
                HighlightStyle::DebugHighlight.style()
            }
            log::Level::Trace => {
                HighlightStyle::TraceHighlight.style()
            }
        };

        let ts = buf.timestamp_millis();
        let lvl = record.level();
        let args = record.args();

        writeln!(
            buf,
            "[{TIMESTAMP_STYLE}{}{TIMESTAMP_STYLE:#}][{level_colour}{}{level_colour:#}][{THREAD_NAME_STYLE}{}{THREAD_NAME_STYLE:#}] {DEFAULT_STYLE}{}{DEFAULT_STYLE:#}",
            ts,
            lvl,
            thread::current().name().unwrap_or_default().to_ascii_uppercase(),
            args
        )
    }).init();
}

pub const DARK_GREY_HIGHLIGHT: Style = Style::new().fg_color(Some(Color::Ansi256(Ansi256Color(8))));
pub const RED_HIGHLIGHT: Style = Style::new().fg_color(Some(Color::Ansi256(Ansi256Color(9))));
pub const GREEN_HIGHLIGHT: Style = Style::new().fg_color(Some(Color::Ansi256(Ansi256Color(10))));
pub const YELLOW_HIGHLIGHT: Style = Style::new().fg_color(Some(Color::Ansi256(Ansi256Color(11))));
pub const BLUE_HIGHLIGHT: Style = Style::new().fg_color(Some(Color::Ansi256(Ansi256Color(12))));
pub const PURPLE_HIGHLIGHT: Style = Style::new().fg_color(Some(Color::Ansi256(Ansi256Color(13))));
pub const AQUA_HIGHLIGHT: Style = Style::new().fg_color(Some(Color::Ansi256(Ansi256Color(14))));
pub const DEFAULT_STYLE: Style = BLUE_HIGHLIGHT;
pub const TIMESTAMP_STYLE: Style = DARK_GREY_HIGHLIGHT.underline();
pub const THREAD_NAME_STYLE: Style = AQUA_HIGHLIGHT.bold();

pub enum HighlightStyle {
    TraceHighlight,
    DebugHighlight,
    InfoHighlight,
    WarnHighLight,
    ErrorHighlight,
}

impl HighlightStyle {
    pub fn style(&self) -> Style {
        match self {
            HighlightStyle::TraceHighlight => PURPLE_HIGHLIGHT.bold(),
            HighlightStyle::DebugHighlight => GREEN_HIGHLIGHT.bold(),
            HighlightStyle::InfoHighlight => BLUE_HIGHLIGHT.bold(),
            HighlightStyle::WarnHighLight => YELLOW_HIGHLIGHT.bold(),
            HighlightStyle::ErrorHighlight => RED_HIGHLIGHT.bold()
        }
    }
}