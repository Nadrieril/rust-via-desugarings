//@ known-failure
fn foo((a, b, Ok(x) | Err(x)): (u32, u32, Result<bool, bool>)) -> u32 {
    a + b + x as u32
}
