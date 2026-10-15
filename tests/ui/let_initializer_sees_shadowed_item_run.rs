//@ run
//@ known-failure
fn f1() {
    print(true);
}

fn main() {
    let f1: () = f1();
}
