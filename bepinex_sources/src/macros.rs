#[macro_export]
macro_rules! s_parse {
    ($sel:expr) => {
        Selector::parse($sel).unwrap()
    };
}

#[macro_export]
macro_rules! select {
    ($elem:expr, $sel:expr) => {
        $elem.select($sel).next().unwrap()
    };
}
