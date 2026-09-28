use std::fmt;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, format::Writer};
use tracing_subscriber::registry::LookupSpan;

pub struct ShortTarget;

impl<S, N> FormatEvent<S, N> for ShortTarget
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> fmt::Result {
        let meta = event.metadata();
        let target = meta.target();
        let short = target
            .strip_prefix("omikuji_core::")
            .or_else(|| target.strip_prefix("omikuji::"))
            .unwrap_or(target);

        let level = meta.level();
        let (color, reset) = if writer.has_ansi_escapes() {
            (
                match *level {
                    tracing::Level::ERROR => "\x1b[31m",
                    tracing::Level::WARN => "\x1b[33m",
                    tracing::Level::INFO => "\x1b[32m",
                    tracing::Level::DEBUG => "\x1b[34m",
                    tracing::Level::TRACE => "\x1b[35m",
                },
                "\x1b[0m",
            )
        } else {
            ("", "")
        };

        write!(writer, "{color}{level:>5}{reset} {short}: ")?;
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}
