extern crate rand;
extern crate noisy_float;

use rand::distributions::{Distribution, Range};
use rand::thread_rng;
use noisy_float::checkers::NumChecker;
use noisy_float::NoisyFloat;

pub trait Mutate: Sized {
    type Database;
    fn new_database() -> Self::Database;
    fn insert_in_database(&self, database: &mut Self::Database);
    fn mutate_in_database(&self, database: &Self::Database, distribution: &impl Distribution<f64>) -> Self;
}

macro_rules! impl_ord_primitive {
    ( $( $t:ty),* ) => {
        $(
impl Mutate for $t {
    type Database = Vec<$t>;
    fn new_database() -> Self::Database {
        vec![]
    }
    fn insert_in_database(&self, database: &mut Self::Database) {
        database.push(*self);
        database.sort_unstable();
    }
    fn mutate_in_database(&self, database: &Self::Database, distribution: &impl Distribution<f64>) -> Self {
        let mut rng = thread_rng();
        let indices = database.iter()
            .enumerate()
            .skip_while(|(_, &value)| value != *self)
            .take_while(|(_, &value)| value == *self)
            .map(|(indice, _)| indice)
            .collect::<Vec<_>>();

        let mut indice = indices[Range::new(0, indices.len()).sample(&mut rng)];
        indice = (indice as isize + distribution.sample(&mut rng).round() as isize).max(0) as usize;
        indice = indice.min(database.len() - 1);
        database[indice]
    }
}
        )*
    };
}

impl_ord_primitive!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

macro_rules! impl_float_primitive {
    ( $( $t:ty),* ) => {
        $(
impl Mutate for $t {
    type Database = Vec<NoisyFloat<$t, NumChecker>>;
    fn new_database() -> Self::Database {
        vec![]
    }
    fn insert_in_database(&self, database: &mut Self::Database) {
        if let Some(value) = NoisyFloat::try_new(*self) {
            database.push(value);
            database.sort_unstable();
        }
    }
    fn mutate_in_database(&self, database: &Self::Database, distribution: &impl Distribution<f64>) -> Self {
        let mut rng = thread_rng();
        let indices = database.iter()
            .enumerate()
            .skip_while(|(_, &value)| value != *self)
            .take_while(|(_, &value)| value == *self)
            .map(|(indice, _)| indice)
            .collect::<Vec<_>>();

        let mut indice = indices[Range::new(0, indices.len()).sample(&mut rng)];
        indice = (indice as isize + distribution.sample(&mut rng).round() as isize).max(0) as usize;
        indice = indice.min(database.len() - 1);
        database[indice].raw()
    }
}
        )*
    };
}

impl_float_primitive!(f32, f64);

impl<T: Mutate> Mutate for Vec<T> {
    type Database = <T as Mutate>::Database;
    fn new_database() -> Self::Database {
        <T as Mutate>::new_database()
    }
    fn insert_in_database(&self, database: &mut Self::Database) {
        for value in self {
            value.insert_in_database(database);
        }
    }
    fn mutate_in_database(&self, database: &Self::Database, distribution: &impl Distribution<f64>) -> Self {
        self.iter()
            .map(|value| value.mutate_in_database(database, distribution))
            .collect()
    }
}

impl Mutate for bool {
    // true/false
    type Database = (usize, usize);
    fn new_database() -> Self::Database {
        (0, 0)
    }
    fn insert_in_database(&self, database: &mut Self::Database) {
        if *self {
            database.0 += 1;
        } else {
            database.1 += 1;
        }
    }
    fn mutate_in_database(&self, database: &Self::Database, distribution: &impl Distribution<f64>) -> Self {
        let mut rng = thread_rng();
        let indice_range = if *self {
            Range::new(0, database.0)
        } else {
            Range::new(database.0, database.0 + database.1)
        };

        let mut indice = indice_range.sample(&mut rng);
        indice = (indice as isize + distribution.sample(&mut rng).round() as isize).max(0) as usize;
        indice = indice.min(database.0 + database.1);
        indice < database.0
    }
}
