#![allow(dead_code)]

use my_derive::Getters;

#[derive(Getters)]
struct MyStructRef<'a> {
    /// 你好呀
    #[getter(vis=pub(crate))]
    #[getter(name = "get_fuck_data")]
    data: &'a str,
}

fn main() {}
