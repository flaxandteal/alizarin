#[allow(dead_code)]
pub fn set_panic_hook() {
    // Call this function at least once during initialization to get
    // better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    console_error_panic_hook::set_once();
}
