use std::fmt::Debug;
use std::fmt::Display;
use std::ops::Deref;

pub struct MagicError {
    inner: ErrorImpl,
}

struct ErrorImpl {
    inner: Box<dyn std::error::Error + Send + Sync>,
}

impl Debug for MagicError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        Debug::fmt(&self.inner, formatter)
    }
}

impl Display for MagicError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(&self.inner, formatter)
    }
}

impl<E> From<E> for ErrorImpl
where
    E: std::error::Error + Send + Sync + 'static,
{
    #[cold]
    fn from(error: E) -> Self {
        ErrorImpl {
            inner: Box::new(error),
        }
    }
}

impl From<MagicError> for Box<dyn std::error::Error + Send + Sync + 'static> {
    fn from(value: MagicError) -> Self {
        value.inner.inner
    }
}

impl From<MagicError> for Box<dyn std::error::Error + Send + 'static> {
    fn from(value: MagicError) -> Self {
        value.inner.inner
    }
}

impl From<MagicError> for Box<dyn std::error::Error + 'static> {
    fn from(value: MagicError) -> Self {
        value.inner.inner
    }
}

impl<E> From<E> for MagicError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(error: E) -> Self {
        MagicError {
            inner: Box::new(error).into(),
        }
    }
}

impl Debug for ErrorImpl {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        Debug::fmt(&self.inner, formatter)
    }
}

impl Display for ErrorImpl {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(&self.inner, formatter)
    }
}

impl Deref for MagicError {
    type Target = dyn std::error::Error + Send + Sync + 'static;

    fn deref(&self) -> &Self::Target {
        self.inner.inner.as_ref()
    }
}

impl AsRef<dyn std::error::Error + Send + Sync> for MagicError {
    fn as_ref(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        &**self
    }
}

impl AsRef<dyn std::error::Error> for MagicError {
    fn as_ref(&self) -> &(dyn std::error::Error + 'static) {
        &**self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_magic_migrate_error() {
        #[derive(Debug, thiserror::Error)]
        enum Error {
            #[error("Ohno: {0}")]
            OhNo(String),
        }

        let error: MagicError = Error::OhNo("An error".to_string()).into();
        let _: Box<dyn std::error::Error> = error.into();
    }
}
