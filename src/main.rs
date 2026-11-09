fn main() {
    let code = course_integrity::cli::main(std::env::args_os());
    if code != 0 {
        std::process::exit(code);
    }
}
