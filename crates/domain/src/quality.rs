/// Codecs that keep every bit of the original audio.
const LOSSLESS_CODECS: &[&str] = &["FLAC", "ALAC", "WAV", "AIFF", "PCM", "DSD", "DSF"];

/// The Hi-Res rule: lossless, at least 24 bits and at least 44.1 kHz.
/// This follows the labelling of 24-bit streaming catalogues; the stricter
/// industry definition starts at 96 kHz.
const HI_RES_MIN_BIT_DEPTH: u32 = 24;
const HI_RES_MIN_SAMPLE_RATE_HZ: u32 = 44_100;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AudioFormat {
    pub codec: String,
    pub bit_depth: u32,
    pub sample_rate_hz: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QualityMeasure {
    Format(AudioFormat),
    /// Only the bitrate is known, which implies a lossy stream.
    Bitrate {
        bits_per_second: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AudioQuality {
    pub measure: QualityMeasure,
    /// Extra information from the source, e.g. "CD quality".
    pub detail: Option<String>,
}

impl AudioQuality {
    pub fn format(format: AudioFormat) -> Self {
        Self {
            measure: QualityMeasure::Format(format),
            detail: None,
        }
    }

    pub fn bitrate(bits_per_second: u32) -> Self {
        Self {
            measure: QualityMeasure::Bitrate { bits_per_second },
            detail: None,
        }
    }

    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// "FLAC 24/96" for a format, "160 kbps" for a bitrate.
    pub fn label(&self) -> String {
        match &self.measure {
            QualityMeasure::Format(format) => format!(
                "{} {}/{}",
                format.codec.to_uppercase(),
                format.bit_depth,
                kilohertz(format.sample_rate_hz)
            ),
            QualityMeasure::Bitrate { bits_per_second } => {
                format!("{} kbps", (bits_per_second + 500) / 1000)
            }
        }
    }

    pub fn is_lossless(&self) -> bool {
        match &self.measure {
            QualityMeasure::Format(format) => LOSSLESS_CODECS
                .iter()
                .any(|codec| codec.eq_ignore_ascii_case(&format.codec)),
            QualityMeasure::Bitrate { .. } => false,
        }
    }

    pub fn is_hi_res(&self) -> bool {
        match &self.measure {
            QualityMeasure::Format(format) => {
                self.is_lossless()
                    && format.bit_depth >= HI_RES_MIN_BIT_DEPTH
                    && format.sample_rate_hz >= HI_RES_MIN_SAMPLE_RATE_HZ
            }
            QualityMeasure::Bitrate { .. } => false,
        }
    }
}

/// 96000 -> "96", 44100 -> "44.1", 22050 -> "22.05".
fn kilohertz(hz: u32) -> String {
    let whole = hz / 1000;
    let fraction = format!("{:03}", hz % 1000);
    let fraction = fraction.trim_end_matches('0');
    if fraction.is_empty() {
        whole.to_string()
    } else {
        format!("{whole}.{fraction}")
    }
}
