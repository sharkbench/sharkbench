pub fn panic_with_stacktrace(message: &str) {
    let backtrace = std::backtrace::Backtrace::capture();
    panic!("{}\n{:?}", message, backtrace);
}
