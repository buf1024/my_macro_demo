#![allow(dead_code)]

use my_derive::Getters;

#[derive(Getters)]
enum MyEnum {
    #[getter(vis=pub(crate))]
    #[getter(name = "get_fuck_data")]
    Data,
}

fn main() {}
