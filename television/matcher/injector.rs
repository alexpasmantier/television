/// An injector that can be used to push items of type `I` into the fuzzy matcher.
///
/// This is a wrapper around the `Injector` type from the `Nucleo` fuzzy matcher.
///
/// The `push` method takes an item of type `I` and a closure that produces the
/// string to match against based on the item.
#[derive(Clone)]
pub struct Injector<I>
where
    I: Sync + Send + Clone + 'static,
{
    /// The inner `Injector` from the `Nucleo` fuzzy matcher.
    inner: nucleo::Injector<I>,
}

impl<I> Injector<I>
where
    I: Sync + Send + Clone + 'static,
{
    pub fn new(inner: nucleo::Injector<I>) -> Self {
        Self { inner }
    }

    /// Push an item into the fuzzy matcher.
    ///
    /// The closure `f` should produce the string to match against based on the
    /// item.
    ///
    /// # Example
    /// ```
    /// use television::matcher::{config::Config, Matcher};
    ///
    /// let config = Config::default();
    /// let matcher = Matcher::new(&config);
    ///
    /// let injector = matcher.injector();
    /// injector.push(
    ///     ("some string", 3, "some other string"),
    ///     // Say we want to match against the third element of the tuple
    ///     |s, cols| cols[0] = s.2.into()
    /// );
    /// ```
    pub fn push<F>(&self, item: I, f: F)
    where
        F: FnOnce(&I, &mut [nucleo::Utf32String]),
    {
        self.inner.push(item, f);
    }
}
