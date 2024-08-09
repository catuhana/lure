use tracing::{trace, Level, Metadata};
use tracing_subscriber::{
    fmt::{self, MakeWriter},
    layer::SubscriberExt,
    EnvFilter,
};

struct StdWriter {
    stdout: std::io::Stdout,
    stderr: std::io::Stderr,
}

enum StdIOLock<'a> {
    Stdout(std::io::StdoutLock<'a>),
    Stderr(std::io::StderrLock<'a>),
}

impl StdWriter {
    fn new() -> Self {
        Self {
            stdout: std::io::stdout(),
            stderr: std::io::stderr(),
        }
    }
}

impl<'a> std::io::Write for StdIOLock<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Stdout(stdout) => stdout.write(buf),
            Self::Stderr(stderr) => stderr.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Stdout(stdout) => stdout.flush(),
            Self::Stderr(stderr) => stderr.flush(),
        }
    }
}

impl<'a> MakeWriter<'a> for StdWriter {
    type Writer = StdIOLock<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        StdIOLock::Stdout(self.stdout.lock())
    }

    fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer {
        if meta.level() <= &Level::WARN {
            return StdIOLock::Stderr(self.stderr.lock());
        }

        StdIOLock::Stdout(self.stdout.lock())
    }
}

pub fn set_up() -> anyhow::Result<()> {
    let std_writer = StdWriter::new();

    let collector =
        tracing_subscriber::registry()
            .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                EnvFilter::new(format!("{}=info", std::env!("CARGO_PKG_NAME")))
            }))
            .with(fmt::Layer::new().with_writer(std_writer));

    tracing::subscriber::set_global_default(collector)?;

    trace!("initialised tracing");

    Ok(())
}
