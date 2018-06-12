#[macro_use]
extern crate mutate_derive;
extern crate mutate;
extern crate rand;

#[derive(CompositeMutate)]
pub struct A(f32, f32);
#[derive(CompositeMutate)]
pub struct B {
    #[mutate(skip)]
    d: String,
    a: f32,
    b: f32,
}

#[repr(usize)]
#[derive(EnumMutate)]
pub enum C {
    A,
    B,
    C,
}
