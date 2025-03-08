fn main() {
    if let Err(e) = fzweb::get_args().and_then(fzweb::run) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
