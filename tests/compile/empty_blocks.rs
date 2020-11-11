#[spandoc::spandoc]
fn main() {}

#[spandoc::spandoc]
fn _panic_no_return_type() {}

#[spandoc::spandoc]
fn _panic_match_empty_block() {
    match 42 {
        _ => {}
    };
}

#[spandoc::spandoc]
fn _dont_panic_return_type_value() -> i32 {
    42
}

#[spandoc::spandoc]
fn _dont_panic_no_return_type_but_unit_value() {
    ()
}

#[spandoc::spandoc]
fn _dont_panic_match_non_empty_block() {
    match 42 {
        _ => 42,
    };
}
