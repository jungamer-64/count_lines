use crate::domain::config::Config;
use std::io::Write;

pub(crate) struct OutputWriter(Box<dyn Write>);

impl OutputWriter {
    pub(crate) fn create(config: &Config) -> anyhow::Result<Self> {
        let writer: Box<dyn Write> = if let Some(path) = &config.output {
            Box::new(std::io::BufWriter::new(std::fs::File::create(path)?))
        } else {
            Box::new(std::io::BufWriter::new(std::io::stdout()))
        };
        Ok(Self(writer))
    }
}

impl Write for OutputWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}
