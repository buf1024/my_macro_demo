#![allow(dead_code)]

use my_derive::{formula, retry, Getters};

#[derive(Getters)]
struct MyStruct {
    data: String,
}

#[derive(Getters)]
struct MyStructRef<'a> {
    /// 你好呀
    #[getter(vis=pub(crate))]
    #[getter(name = "get_fuck_data")]
    data: &'a str,
}
#[derive(Getters)]
struct MyStruct2<'a, T: Sync + Send> {
    #[getter(vis=pub(crate))]
    #[getter(name=get_fuck_data)]
    data: &'a str,
    aa: T,
}
#[retry(times = 5, timeout = 60)]
fn remote_request(a: i32, b: i32) -> i32 {
    println!("@remote_request!");
    a + b
}

fn main() {
    let m = MyStruct {
        data: "my data".into(),
    };

    println!("MyStruct: {}", m.get_data());

    let m = MyStructRef {
        data: "my data ref",
    };

    println!("MyStructRef: {}", m.get_fuck_data());

    let (x, y) = formula!(1 * x + 1 * y = 2, 2 * x + 1 * y = 9);
    println!("x = {}, y = {}", x, y)
}
