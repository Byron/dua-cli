use dua::{ByteFormat, WalkOptions};
use std::fmt;

#[derive(Clone, Copy)]
pub enum ByteVisualization {
    Percentage,
    Bar,
    LongBar,
    PercentageAndBar,
}

pub struct DisplayByteVisualization {
    format: ByteVisualization,
    percentage: f32,
}

impl Default for ByteVisualization {
    fn default() -> Self {
        ByteVisualization::PercentageAndBar
    }
}

impl ByteVisualization {
    pub fn cycle(&mut self) {
        use ByteVisualization::*;
        *self = match self {
            Bar => LongBar,
            LongBar => PercentageAndBar,
            PercentageAndBar => Percentage,
            Percentage => Bar,
        }
    }
    pub fn display(&self, percentage: f32) -> DisplayByteVisualization {
        DisplayByteVisualization {
            format: *self,
            percentage,
        }
    }
}

impl fmt::Display for DisplayByteVisualization {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use ByteVisualization::*;
        let Self { format, percentage } = self;

        const BAR_SIZE: usize = 10;
        match format {
            Percentage => Self::make_percentage(f, percentage),
            PercentageAndBar => {
                Self::make_percentage(f, percentage)?;
                f.write_str(" ")?;
                Self::make_bar(f, percentage, BAR_SIZE)
            }
            Bar => Self::make_bar(f, percentage, BAR_SIZE),
            LongBar => Self::make_bar(f, percentage, 20),
        }
    }
}

impl DisplayByteVisualization {
    fn make_bar(f: &mut fmt::Formatter, percentage: &f32, length: usize) -> Result<(), fmt::Error> {
        let block_length = (length as f32 * percentage).round() as usize;
        for _ in 0..block_length {
            f.write_str(tui::symbols::block::FULL)?;
        }
        for _ in 0..length - block_length {
            f.write_str(" ")?;
        }
        Ok(())
    }
    fn make_percentage(f: &mut fmt::Formatter, percentage: &f32) -> Result<(), fmt::Error> {
        write!(f, " {:>5.02}% ", percentage * 100.0)
    }
}

/// Options to configure how we display things
#[derive(Clone, Copy)]
pub struct DisplayOptions {
    pub byte_format: ByteFormat,
    pub byte_vis: ByteVisualization,
}

impl From<WalkOptions> for DisplayOptions {
    fn from(WalkOptions { byte_format, .. }: WalkOptions) -> Self {
        DisplayOptions {
            byte_format,
            byte_vis: ByteVisualization::default(),
        }
    }
}
