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

    pub fn class(&self) -> QualityClass {
        if self.is_hi_res() {
            QualityClass::HiRes
        } else if self.is_lossless() {
            QualityClass::Lossless
        } else {
            QualityClass::Lossy
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

/// How good a path's audio is, from worst to best: the order lets the
/// class of a path be the lowest of its parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum QualityClass {
    /// No quality reported.
    Unknown,
    Lossy,
    /// Lossless, but not Hi-Res (CD quality, for example).
    Lossless,
    HiRes,
}

/// What a route (Music Assistant, for example) sends on to the player.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Delivery {
    /// The quality the player receives; `None` when the route doesn't say.
    pub quality: Option<AudioQuality>,
    /// What the route changed on the way, in this order: resampling, bit
    /// depth, normalisation, DSP.
    pub changes: Vec<Change>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Change {
    /// The sample rate differs from the source's.
    Resampled,
    /// Only the bit depth differs from the source's.
    BitDepth,
    /// The loudness was normalised, by this gain when it is reported.
    Normalised { gain: Option<Decibels> },
    /// The route's DSP (equaliser, filters) is on.
    Dsp,
}

/// A gain, kept in tenths of a decibel so it compares exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Decibels(i32);

impl Decibels {
    #[allow(clippy::cast_possible_truncation)] // gains are a few dozen dB
    pub fn from_db(db: f64) -> Self {
        Self((db * 10.0).round() as i32)
    }

    pub fn as_db(self) -> f64 {
        f64::from(self.0) / 10.0
    }
}

/// The class of a path: the lowest of the source's and the delivered
/// quality. Upsampling never raises it; without a source quality it is
/// unknown.
pub fn signal_class(source: Option<&AudioQuality>, delivery: Option<&Delivery>) -> QualityClass {
    let Some(source) = source else {
        return QualityClass::Unknown;
    };
    let source_class = source.class();
    match delivery.and_then(|delivery| delivery.quality.as_ref()) {
        Some(delivered) => source_class.min(delivered.class()),
        None => source_class,
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
