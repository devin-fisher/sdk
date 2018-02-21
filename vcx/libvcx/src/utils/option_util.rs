pub fn expect_ok_or<T, E>(opt: Option<T>, none_warning: &str, error_val: E) -> Result<T, E> {
    opt.ok_or_else(||{
        warn!("Option was None: {}", none_warning);
        error_val
    })
}

pub fn ok_or<T, E>(opt: Option<T>, missing_msg: &str, error_val: E) -> Result<T, E> {
    opt.ok_or(error_val)
}