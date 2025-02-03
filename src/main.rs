use std::env;
use pointeuse2::punch_orhis;

// usage :  ./pointeuse2 loginCAS passwordCAS
fn main() {
    let args = env::args().collect::<Vec<String>>();
    let login = &args[1];
    let password = &args[2];
    punch_orhis(login, password);
}
