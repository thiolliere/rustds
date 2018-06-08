extern crate lmms_drum_synth_sys;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_ini;
extern crate regex;
extern crate tempfile;
extern crate failure;
extern crate byteorder;

use std::fs::File;
use std::ffi::CString;
use serde::{Deserializer, Deserialize, Serializer};
use serde::de;
use regex::Regex;
use std::path::Path;
use std::io::Read;

/// (time-level, point)
pub type Envelope = Vec<(i32, i32)>;

fn deserialize_point_list_from_str<'de, D>(deserializer: D) -> Result<Vec<(i32, i32)>, D::Error>
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
        list.push((
            i32::from_str_radix(&caps[1], 10)
                .map_err(de::Error::custom)?,
            i32::from_str_radix(&caps[2], 10)
                .map_err(de::Error::custom)?,
        ));
    }

    Ok(list)
}

fn serialize_point_list_to_str<S>(envelope: &Envelope, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer
{
    let mut string = String::new();
    for (x, y) in envelope {
        string.push_str(&format!("{},{} ", x, y));
    }

    // remove trailing space
    string.pop();

    serializer.serialize_str(&string)
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

fn deserialize_invalid_utf8_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>
{
    let b: String = Deserialize::deserialize(deserializer)
        .unwrap_or_else(|_| String::new());
    Ok(Some(b))
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct DrumSynth {
    General: General,
    // Tone: Tone,
    // Noise: Noise,
    // Overtones: Overtones,
    // NoiseBand: NoiseBand,
    // NoiseBand2: NoiseBand2,
    // Distortion: Distortion,
}

fn default_100f32() -> f32 {
    100f32
}

fn default_0_0_100_0() -> Vec<(i32, i32)> {
    vec![
        (0, 0),
        (100, 0),
    ]
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct General {
    Version: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_invalid_utf8_string")]
    #[serde(skip_serializing_if = "Option::is_none")]
    Comment: Option<String>,
    #[serde(default)]
    Tuning: f32,
    #[serde(default = "default_100f32")]
    Stretch: f32,
    #[serde(default)]
    Filter: i32,
    #[serde(default)]
    HighPass: i32,
    #[serde(deserialize_with = "deserialize_point_list_from_str")]
    #[serde(serialize_with = "serialize_point_list_to_str")]
    #[serde(default = "default_0_0_100_0")]
    FilterEnv: Vec<(i32, i32)>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Tone {
    #[serde(deserialize_with = "deserialize_bool_from_int")]
    #[serde(serialize_with = "serialize_bool_to_dsint")]
    On: bool,
    Level: i32,
    F1: i32,
    F2: i32,
    Droop: i32,
    Phase: i32,
    #[serde(deserialize_with = "deserialize_point_list_from_str")]
    #[serde(serialize_with = "serialize_point_list_to_str")]
    Envelope: Vec<(i32, i32)>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Noise {
    #[serde(deserialize_with = "deserialize_bool_from_int")]
    #[serde(serialize_with = "serialize_bool_to_dsint")]
    On: bool,
    Level: i32,
    Slope: i32,
    #[serde(deserialize_with = "deserialize_point_list_from_str")]
    #[serde(serialize_with = "serialize_point_list_to_str")]
    Envelope: Vec<(i32, i32)>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Overtones {
    #[serde(deserialize_with = "deserialize_bool_from_int")]
    #[serde(serialize_with = "serialize_bool_to_dsint")]
    On: bool,
    Level: i32,
    F1: i32,
    Wave1: i32,
    Track1: i32,
    F2: i32,
    Wave2: i32,
    Track2: i32,
    Filter: i32,
    Method: i32,
    Param: i32,
    #[serde(deserialize_with = "deserialize_point_list_from_str")]
    #[serde(serialize_with = "serialize_point_list_to_str")]
    Envelope1: Vec<(i32, i32)>,
    #[serde(serialize_with = "serialize_point_list_to_str")]
    #[serde(deserialize_with = "deserialize_point_list_from_str")]
    Envelope2: Vec<(i32, i32)>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct NoiseBand {
    #[serde(deserialize_with = "deserialize_bool_from_int")]
    #[serde(serialize_with = "serialize_bool_to_dsint")]
    On: bool,
    Level: i32,
    F: i32,
    dF: i32,
    #[serde(deserialize_with = "deserialize_point_list_from_str")]
    #[serde(serialize_with = "serialize_point_list_to_str")]
    Envelope: Vec<(i32, i32)>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct NoiseBand2 {
    #[serde(deserialize_with = "deserialize_bool_from_int")]
    #[serde(serialize_with = "serialize_bool_to_dsint")]
    On: bool,
    Level: i32,
    F: i32,
    dF: i32,
    #[serde(deserialize_with = "deserialize_point_list_from_str")]
    #[serde(serialize_with = "serialize_point_list_to_str")]
    Envelope: Vec<(i32, i32)>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Distortion {
    #[serde(deserialize_with = "deserialize_bool_from_int")]
    #[serde(serialize_with = "serialize_bool_to_dsint")]
    On: bool,
    Clipping: i32,
    Bits: i32,
    Rate: i32,
}

impl DrumSynth {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, failure::Error> {
        let mut file = File::open(path)?;
        let mut content = vec![];
        file.read_to_end(&mut content);
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

fn main() {
}

#[cfg(test)]
mod test {
    use std::path::Path;
    use std::io::{self, Read};
    use std::fs::{self, DirEntry, File};
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
    fn samples() {
        visit_dirs(Path::new("tests/lmms_drum_synth_samples"), &|entry: &DirEntry| {
            let res = DrumSynth::load(entry.path());
            if res.is_err() {
                println!("{}", entry.path().to_string_lossy());
            }
            res.unwrap();
        }).unwrap();
    }
}
