use rand::distributions::{Distribution, Range};
use rand::thread_rng;
use std::collections::VecDeque;

impl Toto {
    fn finalize(&mut self) {
        self.General.Stretch = self.General.Stretch.max(0.2).min(10.0);
    }
    fn length(&self) -> i32 {
        // Doit on prendre en compte filterEnv ?
        // Pour ¿certaine: 1 à 6 compris? enveloppe active prendre le time-level le plus élevé
    }
    fn loudest(&self) -> i32 {
    }
}


fn get_ds_file_samples(ds: &DrumSynth, wave: &mut [i16], channels: i32, sample_rate: i32) {
    let mut rng = thread_rng();

    let master_tune = 0.0; // TODO

    let mut noise_rand = VecDeque::new();
    noise_rand.resize(3, 0.0);
    let mut noise_range = Range::new(0.0, 1.0); // TODO: randmax2
    let mut noise_g = 1.0; // TODO: g
    let mut noise_tt = 0.0;
    let mut noise_weight = [0.0; 4]; // TODO: a, b, c, d

    let mut f1 = master_tune * 2.0 * PI * ds.Tone.F1;
    if f1.abs() < 0.001 {
        f1 = 0.001
    }
    let mut df = [0.0; 1000];
    let mut phi = [0.0; 1000];

    for t in 0..1000 {
        if ds.Noise.On {
            noise_rand.pop_front();
            noise_rand.push_back(noise_range.sample(&mut rng));

            noise_tt = noise_weight[0] * noise_rand[3]
                + noise_weight[1] * noise_rand[2]
                + noise_weight[2] * noise_rand[1]
                + noise_weight[3] * noise_tt;

            df[t] += noise_tt * noise_g; //* envDat[2][ENV]
        }

        if ds.Tone.On {
            if ds.Tone.Droop > 0.0 {
            }
        } else {
        }
    }
}

