pub trait WebSafeExpect<T> {
    fn web_expect(self, msg: &str) -> T;
}

impl<T> WebSafeExpect<T> for Option<T> {
    fn web_expect(self, msg: &str) -> T {
        match self {
            Some(val) => val,
            None => {
                log::error!("{}", msg);
                log::logger().flush();
                panic!()
            }
        }
    }
}

impl<T, E: std::fmt::Debug> WebSafeExpect<T> for Result<T, E> {
    fn web_expect(self, msg: &str) -> T {
        match self {
            Ok(val) => val,
            Err(e) => {
                log::error!("{} - {:?}", msg, e);
                log::logger().flush();
                panic!()
            }
        }
    }
}
