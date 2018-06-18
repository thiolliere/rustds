extern crate rds;
extern crate clap;
extern crate hound;

use clap::{App, Arg};

fn main() -> Result<(), String> {
    // TODO: a subcommand for mutation
    let matches = App::new("rds-cli")
        .arg(Arg::with_name("sample-rate")
             .short("r")
             .long("sample-rate")
             .value_name("NUMBER")
             .default_value("44100"))
        .arg(Arg::with_name("channels")
             .short("c")
             .long("channels")
             .value_name("NUMBER")
             .default_value("1"))
        .arg(Arg::with_name("input")
             .short("i")
             .long("input")
             .value_name("FILE")
             .required(true))
        .arg(Arg::with_name("output")
             .short("o")
             .long("output")
             .value_name("FILE")
             .default_value("out.wav"))
        .get_matches();

    let ds = rds::DrumSynth::load(matches.value_of("input").unwrap())
        .map_err(|e| format!("Cannot load ds file: {}", e))?;
    let channels = u16::from_str_radix(matches.value_of("channels").unwrap(), 10)
        .map_err(|e| format!("Invalid channel: {}", e))?;
    let sample_rate = u32::from_str_radix(matches.value_of("sample-rate").unwrap(), 10)
        .map_err(|e| format!("Invalid sample rate: {}", e))?;
    let data = ds.get_ds_file_samples(channels as i32, sample_rate)
        .map_err(|e| format!("Failed to get samples: {}", e))?;

    let spec = hound::WavSpec {
        channels: channels as u16,
        sample_rate: sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(matches.value_of("output").unwrap(), spec)
        .map_err(|e| format!("Failed to write output: {}", e))?;

    for &mut sample in data {
        writer.write_sample(sample)
            .map_err(|e| format!("Failed to write output: {}", e))?;
    }
    writer.finalize()
        .map_err(|e| format!("Failed to write output: {}", e))?;
    Ok(())
}
