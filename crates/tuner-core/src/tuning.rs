use crate::{error::positive_finite, CoreError};

// Only resolves floating-point roundoff at equal-distance boundaries (1e-8 cent).
const MIDI_TIE_EPSILON: f64 = 1e-10;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MidiNote(u8);

impl MidiNote {
    pub fn new(number: i32) -> Result<Self, CoreError> {
        if !(0..=127).contains(&number) {
            return Err(CoreError::MidiOutOfRange);
        }
        Ok(Self(number as u8))
    }

    pub const fn number(self) -> u8 {
        self.0
    }
    pub const fn octave(self) -> i8 {
        (self.0 / 12) as i8 - 1
    }

    pub fn pitch_class(self, spelling: NoteSpelling) -> &'static str {
        const SHARPS: [&str; 12] = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        const FLATS: [&str; 12] = [
            "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
        ];
        match spelling {
            NoteSpelling::Sharps => SHARPS[(self.0 % 12) as usize],
            NoteSpelling::Flats => FLATS[(self.0 % 12) as usize],
        }
    }

    pub fn name(self, spelling: NoteSpelling) -> String {
        format!("{}{}", self.pitch_class(spelling), self.octave())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteSpelling {
    Sharps,
    Flats,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TuningDirection {
    Flat,
    InTune,
    Sharp,
}

#[derive(Clone, Copy, Debug)]
pub enum TargetSelection<'a> {
    Chromatic,
    Locked(MidiNote),
    Nearest(&'a [MidiNote]),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TuningResult {
    midi_note: MidiNote,
    note_name: String,
    target_hz: f64,
    cents: f64,
    direction: TuningDirection,
}

impl TuningResult {
    pub const fn midi_note(&self) -> MidiNote {
        self.midi_note
    }
    pub fn note_name(&self) -> &str {
        &self.note_name
    }
    pub const fn target_hz(&self) -> f64 {
        self.target_hz
    }
    pub const fn cents(&self) -> f64 {
        self.cents
    }
    pub const fn direction(&self) -> TuningDirection {
        self.direction
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TuningConfig {
    reference_a4_hz: f64,
    tolerance_cents: f64,
    spelling: NoteSpelling,
}

impl Default for TuningConfig {
    fn default() -> Self {
        Self {
            reference_a4_hz: 440.0,
            tolerance_cents: 3.0,
            spelling: NoteSpelling::Sharps,
        }
    }
}

impl TuningConfig {
    pub fn new(
        reference_a4_hz: f64,
        tolerance_cents: f64,
        spelling: NoteSpelling,
    ) -> Result<Self, CoreError> {
        validate_reference(reference_a4_hz)?;
        if !tolerance_cents.is_finite() || !(0.0..=50.0).contains(&tolerance_cents) {
            return Err(CoreError::InvalidValue("tolerance_cents (0..=50)"));
        }
        Ok(Self {
            reference_a4_hz,
            tolerance_cents,
            spelling,
        })
    }

    pub const fn reference_a4_hz(self) -> f64 {
        self.reference_a4_hz
    }
    pub const fn tolerance_cents(self) -> f64 {
        self.tolerance_cents
    }
    pub const fn spelling(self) -> NoteSpelling {
        self.spelling
    }

    pub fn tune(
        self,
        frequency_hz: f64,
        target: TargetSelection<'_>,
    ) -> Result<TuningResult, CoreError> {
        let fractional_midi = frequency_to_midi(frequency_hz, self.reference_a4_hz)?;
        let midi_note = match target {
            TargetSelection::Chromatic => {
                let lower = fractional_midi.floor();
                let rounded = if (fractional_midi - lower - 0.5).abs() <= MIDI_TIE_EPSILON {
                    lower + 1.0
                } else {
                    fractional_midi.round()
                };
                if !(0.0..=127.0).contains(&rounded) {
                    return Err(CoreError::MidiOutOfRange);
                }
                MidiNote::new(rounded as i32)?
            }
            TargetSelection::Locked(note) => note,
            TargetSelection::Nearest(notes) => {
                let mut best = *notes.first().ok_or(CoreError::EmptyTargets)?;
                let mut distance = (fractional_midi - f64::from(best.number())).abs();
                for &note in &notes[1..] {
                    let next = (fractional_midi - f64::from(note.number())).abs();
                    if next < distance - MIDI_TIE_EPSILON
                        || ((next - distance).abs() <= MIDI_TIE_EPSILON && note > best)
                    {
                        best = note;
                        distance = next;
                    }
                }
                best
            }
        };
        let target_hz = midi_to_frequency(f64::from(midi_note.number()), self.reference_a4_hz)?;
        let cents = cents_difference(frequency_hz, target_hz)?;
        let boundary_epsilon = 100.0 * MIDI_TIE_EPSILON;
        let direction = if cents < -self.tolerance_cents - boundary_epsilon {
            TuningDirection::Flat
        } else if cents > self.tolerance_cents + boundary_epsilon {
            TuningDirection::Sharp
        } else {
            TuningDirection::InTune
        };
        Ok(TuningResult {
            midi_note,
            note_name: midi_note.name(self.spelling),
            target_hz,
            cents,
            direction,
        })
    }
}

fn validate_reference(reference_a4_hz: f64) -> Result<(), CoreError> {
    if !reference_a4_hz.is_finite() || !(400.0..=480.0).contains(&reference_a4_hz) {
        return Err(CoreError::InvalidValue("reference_a4_hz (400..=480)"));
    }
    Ok(())
}

/// Fractional MIDI pitch; positive frequencies may extrapolate beyond 0..=127.
pub fn frequency_to_midi(frequency_hz: f64, reference_a4_hz: f64) -> Result<f64, CoreError> {
    positive_finite(frequency_hz, "frequency_hz")?;
    validate_reference(reference_a4_hz)?;
    Ok(69.0 + 12.0 * (frequency_hz.log2() - reference_a4_hz.log2()))
}

/// Converts a fractional MIDI pitch in 0..=127 to Hz under the chosen A4.
pub fn midi_to_frequency(midi: f64, reference_a4_hz: f64) -> Result<f64, CoreError> {
    validate_reference(reference_a4_hz)?;
    if !midi.is_finite() || !(0.0..=127.0).contains(&midi) {
        return Err(CoreError::MidiOutOfRange);
    }
    Ok(reference_a4_hz * ((midi - 69.0) / 12.0).exp2())
}

/// Unclamped cents. Log subtraction avoids ratio overflow/underflow.
pub fn cents_difference(frequency_hz: f64, target_hz: f64) -> Result<f64, CoreError> {
    positive_finite(frequency_hz, "frequency_hz")?;
    positive_finite(target_hz, "target_hz")?;
    Ok(1200.0 * (frequency_hz.log2() - target_hz.log2()))
}
