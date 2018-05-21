pub static NOTES: &[(usize, &str)] = &[
    (261, "C-4"),
    (293, "D-4"),
    (329, "E-4"),
    (349, "F-4"),
    (391, "G-4"),
    (440, "A-4"),
    (493, "B-4"),
    (523, "C-5"),
];

pub fn frequency(n: usize) -> f64 {
    (2.0f64).powf((n as f64 - 49.0) / 12.0) * 440.0
}

pub fn to_key(f: f64) -> usize {
    (12.0 * (f / 440.0).log2()) as usize + 49
}

static KEYS: &[&str] = &[
    "A", // 1
    "A#",
    "B",
    "C",
    "C#",
    "D",
    "D#",
    "E",
    "F",
    "F#",
    "G",
    "G#", // 12
];

pub fn human_key(n: usize) -> String {
    let octave = n / 12;
    ::log(&format!("{:?}", n));
    format!("{}{}", KEYS[(n % 12) - 1], octave)
}
