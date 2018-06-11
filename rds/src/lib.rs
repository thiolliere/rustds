extern crate lmms_drum_synth_sys;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_ini;
extern crate regex;
extern crate tempfile;
extern crate failure;
extern crate mutate;
#[macro_use]
extern crate mutate_derive;
extern crate rand;

use std::fs::File;
use std::ffi::CString;
use serde::{Deserializer, Deserialize, Serializer};
use serde::de;
use regex::Regex;
use std::path::Path;
use std::io::Read;

pub type Envelope = Vec<Point>;

#[derive(CompositeMutate)]
pub struct Point {
    time_level: i32,
    value: i32,
}

impl Point {
    pub fn new(time_level: i32, value: i32) -> Point {
        Point {
            time_level,
            value,
        }
    }
}

fn deserialize_envelope_from_str<'de, D>(deserializer: D) -> Result<Envelope, D::Error>
where
    D: Deserializer<'de>
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let full_re = Regex::new(r"^(\d+,\d+ )+(\d+,\d+)").unwrap();
    if !full_re.is_match(&s) {
        return Err(de::Error::custom(format!("expect: \"^(\\d+,\\d+ )+(\\d+,\\d+)\", found: \"{}\"", s)));
    }

    let re = Regex::new(r"(\d+),(\d+)").unwrap();
    let mut list = vec![];
    for caps in re.captures_iter(&s) {
        list.push(Point::new(
            i32::from_str_radix(&caps[1], 10)
                .map_err(de::Error::custom)?,
            i32::from_str_radix(&caps[2], 10)
                .map_err(de::Error::custom)?,
        ));
    }

    Ok(list)
}

fn serialize_envelope_to_str<S>(envelope: &Envelope, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer
{
    let mut string = String::new();
    for point in envelope {
        string.push_str(&format!("{},{} ", point.time_level, point.value));
    }

    // remove trailing space
    string.pop();

    serializer.serialize_str(&string)
}

fn default_deserialize_nothing<'a, T>() -> T
where
    T: de::DeserializeOwned,
{
    serde_ini::from_str("").unwrap()
}

fn deserialize_bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>
{
    let n: i32 = Deserialize::deserialize(deserializer)?;

    Ok(n == 1)
}

fn serialize_bool_to_dsint<S>(boolean: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer
{
    let int = if *boolean { 1 } else { 0 };
    serializer.serialize_i32(int)
}

fn default_200f32() -> f32 {
    200f32
}

fn default_120f32() -> f32 {
    120f32
}

fn default_100f32() -> f32 {
    100f32
}

fn default_1000f32() -> f32 {
    1000f32
}

fn default_90f32() -> f32 {
    90f32
}

fn default_0_0_100_0() -> Envelope {
    vec![
        Point::new(0, 0),
        Point::new(100, 0),
    ]
}

fn default_128i32() -> i32 {
    128
}

fn default_2i32() -> i32 {
    2
}

fn default_50i32() -> i32 {
    50
}

#[derive(Serialize, Deserialize, CompositeMutate)]
#[allow(non_snake_case)]
pub struct DrumSynth {
    pub General: General,
    #[serde(default = "default_deserialize_nothing")]
    pub Tone: Tone,
    #[serde(default = "default_deserialize_nothing")]
    pub Noise: Noise,
    #[serde(default = "default_deserialize_nothing")]
    pub Overtones: Overtones,
    #[serde(default = "default_deserialize_nothing")]
    pub NoiseBand: NoiseBand,
    #[serde(default = "default_deserialize_nothing")]
    pub NoiseBand2: NoiseBand,
    #[serde(default = "default_deserialize_nothing")]
    pub Distortion: Distortion,
}

#[derive(Serialize, Deserialize, CompositeMutate)]
#[allow(non_snake_case)]
pub struct General {
    #[mutate(skip)]
    pub Version: String,
    #[mutate(skip)]
    #[serde(default)]
    pub Comment: String,
    #[serde(default)]
    pub Tuning: f32,
    #[serde(default = "default_100f32")]
    pub Stretch: f32,
    #[serde(default)]
    pub Filter: i32,
    #[serde(default)]
    pub HighPass: i32,
    #[serde(deserialize_with = "deserialize_envelope_from_str")]
    #[serde(serialize_with = "serialize_envelope_to_str")]
    #[serde(default = "default_0_0_100_0")]
    pub FilterEnv: Envelope,
}

#[derive(Serialize, Deserialize, CompositeMutate)]
#[allow(non_snake_case)]
pub struct Tone {
    #[serde(deserialize_with = "deserialize_bool_from_int")]
    #[serde(serialize_with = "serialize_bool_to_dsint")]
    #[serde(default)]
    pub On: bool,
    #[serde(default = "default_128i32")]
    pub Level: i32,
    #[serde(default = "default_200f32")]
    pub F1: f32,
    #[serde(default = "default_120f32")]
    pub F2: f32,
    #[serde(default)]
    pub Droop: f32,
    #[serde(default = "default_90f32")]
    pub Phase: f32,
    #[serde(deserialize_with = "deserialize_envelope_from_str")]
    #[serde(serialize_with = "serialize_envelope_to_str")]
    #[serde(default = "default_0_0_100_0")]
    pub Envelope: Envelope,
}


#[derive(Serialize, Deserialize, CompositeMutate)]
#[allow(non_snake_case)]
pub struct Noise {
    #[serde(deserialize_with = "deserialize_bool_from_int")]
    #[serde(serialize_with = "serialize_bool_to_dsint")]
    #[serde(default)]
    pub On: bool,
    #[serde(default)]
    pub Level: i32,
    #[serde(default)]
    pub Slope: i32,
    #[serde(deserialize_with = "deserialize_envelope_from_str")]
    #[serde(serialize_with = "serialize_envelope_to_str")]
    #[serde(default = "default_0_0_100_0")]
    pub Envelope: Envelope,
}

#[derive(Serialize, Deserialize, CompositeMutate)]
#[allow(non_snake_case)]
pub struct Overtones {
    #[serde(deserialize_with = "deserialize_bool_from_int")]
    #[serde(serialize_with = "serialize_bool_to_dsint")]
    #[serde(default)]
    pub On: bool,
    #[serde(default = "default_128i32")]
    pub Level: i32,
    #[serde(default = "default_200f32")]
    pub F1: f32,
    #[serde(default)]
    pub Wave1: i32,
    #[serde(default)]
    pub Track1: i32,
    #[serde(default = "default_120f32")]
    pub F2: f32,
    #[serde(default)]
    pub Wave2: i32,
    #[serde(default)]
    pub Track2: i32,
    #[serde(default)]
    pub Filter: i32,
    #[serde(default = "default_2i32")]
    pub Method: i32,
    #[serde(default = "default_50i32")]
    pub Param: i32,
    #[serde(deserialize_with = "deserialize_envelope_from_str")]
    #[serde(serialize_with = "serialize_envelope_to_str")]
    #[serde(default = "default_0_0_100_0")]
    pub Envelope1: Envelope,
    #[serde(serialize_with = "serialize_envelope_to_str")]
    #[serde(deserialize_with = "deserialize_envelope_from_str")]
    #[serde(default = "default_0_0_100_0")]
    pub Envelope2: Envelope,
}

#[derive(Serialize, Deserialize, CompositeMutate)]
#[allow(non_snake_case)]
pub struct NoiseBand {
    #[serde(deserialize_with = "deserialize_bool_from_int")]
    #[serde(serialize_with = "serialize_bool_to_dsint")]
    #[serde(default)]
    pub On: bool,
    #[serde(default = "default_128i32")]
    pub Level: i32,
    #[serde(default = "default_1000f32")]
    pub F: f32,
    #[serde(default = "default_50i32")]
    pub dF: i32,
    #[serde(deserialize_with = "deserialize_envelope_from_str")]
    #[serde(serialize_with = "serialize_envelope_to_str")]
    #[serde(default = "default_0_0_100_0")]
    pub Envelope: Envelope,
}

#[derive(Serialize, Deserialize, CompositeMutate)]
#[allow(non_snake_case)]
pub struct Distortion {
    #[serde(deserialize_with = "deserialize_bool_from_int")]
    #[serde(serialize_with = "serialize_bool_to_dsint")]
    #[serde(default)]
    pub On: bool,
    #[serde(default)]
    pub Clipping: i32,
    #[serde(default)]
    pub Bits: i32,
    #[serde(default)]
    pub Rate: i32,
}

impl DrumSynth {
    pub fn new() -> Self {
        serde_ini::from_str::<DrumSynth>(&"[General]\nVersion=DrumSynth v2.0").unwrap()
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, failure::Error> {
        let mut file = File::open(path)?;
        let mut content = vec![];
        file.read_to_end(&mut content)?;
        let str_content = String::from_utf8_lossy(&content).into_owned();
        Ok(serde_ini::from_str::<DrumSynth>(&str_content)?)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), failure::Error> {
        let file = File::open(path)?;
        serde_ini::to_writer(file, self)?;
        Ok(())
    }

    pub fn get_ds_file_samples(&self, channels: i32, sample_rate: u32) -> Result<&mut [i16], failure::Error> {
        let wave: &mut [i16];

        let dir = tempfile::tempdir()?;

        let file_path = dir.path().join("tmp.ds");
        let file = File::create(&file_path)?;
        serde_ini::to_writer(file, self)?;

        unsafe {
            let file_path: CString = CString::new(file_path.into_os_string().into_string().unwrap().as_bytes()).unwrap();
            let mut drum_synth = lmms_drum_synth_sys::DrumSynth { _address: 0 };
            let mut buffer: *mut i16 = ::std::mem::uninitialized();
            let length = drum_synth.GetDSFileSamples(file_path.as_ptr() as *const i8, &mut buffer, channels as i32, sample_rate);
            if length == 0 {
                panic!("error getdsfilesamples");
            }
            wave = ::std::slice::from_raw_parts_mut(buffer, length as usize*channels as usize);
        }
        Ok(wave)
    }
}

#[cfg(test)]
mod test {
    use std::path::{Path, PathBuf};
    use std::io;
    use std::fs::{self, DirEntry};
    use std::ffi::OsStr;
    use DrumSynth;

    fn visit_dirs(dir: &Path, cb: &Fn(&DirEntry)) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, cb)?;
                } else {
                    cb(&entry);
                }
            }
        }
        Ok(())
    }
    #[test]
    fn new() {
        DrumSynth::new();
    }

    #[test]
    fn samples() {
        let mut dir = PathBuf::new();
        dir.push(::std::env::var("CARGO_MANIFEST_DIR").unwrap());
        dir.push("..");
        dir.push("lmms-drum-synth-samples");
        assert!(dir.exists());

        visit_dirs(&dir, &|entry: &DirEntry| {
            if let Some(extension) = entry.path().extension() {
                if extension == OsStr::new("ds") {
                    let res = DrumSynth::load(entry.path());
                    if res.is_err() {
                        println!("{}", entry.path().to_string_lossy());
                    }
                    let ds = res.unwrap();

                    let res = ds.get_ds_file_samples(1, 44100);
                    if res.is_err() {
                        println!("{}", entry.path().to_string_lossy());
                    }
                    res.unwrap();
                }
            }
        }).unwrap();
    }
}
