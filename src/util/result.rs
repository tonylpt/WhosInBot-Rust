pub trait ResultExt<T, E> {
    fn on_err<F>(self, on_err_fn: F) -> Result<T, E>
    where
        F: Fn(&E) -> ();
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn on_err<F>(self, on_err_fn: F) -> Result<T, E>
    where
        F: Fn(&E) -> (),
    {
        self.map_err(|err| {
            on_err_fn(&err);
            err
        })
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;

    use super::ResultExt;

    #[test]
    fn test_on_err_with_error() {
        let input: Result<String, String> = Err("error".to_string());
        let captured: RefCell<Option<String>> = RefCell::new(None);
        let actual = input.on_err(|err| {
            captured.borrow_mut().replace(err.clone());
        });

        assert_eq!(Err("error".to_string()), actual);
        assert_eq!("error", captured.borrow().as_ref().unwrap());
    }

    #[test]
    fn test_on_err_with_success() {
        let input: Result<String, String> = Ok("ok".to_string());
        let captured: RefCell<Option<String>> = RefCell::new(None);
        let actual = input.on_err(|err| {
            captured.borrow_mut().replace(err.clone());
        });

        assert_eq!(Ok("ok".to_string()), actual);
        assert!(captured.borrow().is_none());
    }
}
