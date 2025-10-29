// src/infrastructure/io/output/writer.rs
use std::io::{BufWriter, Write};

use crate::{
    domain::config::Config,
    error::{InfrastructureError, Result},
    infrastructure::persistence::FileWriter,
};

pub(crate) struct OutputWriter(Box<dyn Write>);

impl OutputWriter {
    pub(crate) fn create(config: &Config) -> Result<Self> {
        let writer: Box<dyn Write> = if let Some(path) = &config.output {
            Box::new(
                FileWriter::create(path)
                    .map_err(|source| InfrastructureError::FileWrite { path: path.clone(), source })?,
            )
        } else {
            Box::new(BufWriter::new(std::io::stdout()))
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
