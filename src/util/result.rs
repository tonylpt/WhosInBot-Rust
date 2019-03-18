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
