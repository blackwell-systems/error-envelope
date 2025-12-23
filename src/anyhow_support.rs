use crate::{Code, Error};

/// Implements conversion from anyhow::Error to error-envelope::Error.
///
/// This allows seamless integration with anyhow-based error handling:
///
/// ```no_run
/// use error_envelope::Error;
///
/// async fn handler() -> Result<String, Error> {
///     let result = do_work().await?; // anyhow::Error auto-converts
///     Ok(result)
/// }
/// # async fn do_work() -> anyhow::Result<String> { Ok("success".to_string()) }
/// ```
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        // Convert anyhow::Error to internal error with the error message
        Error::new(Code::Internal, 500, err.to_string()).with_retryable(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_anyhow_error() {
        let anyhow_err = anyhow::anyhow!("something went wrong");
        let env_err: Error = anyhow_err.into();

        assert_eq!(env_err.code, Code::Internal);
        assert_eq!(env_err.status(), 500);
        assert_eq!(env_err.message, "something went wrong");
        assert_eq!(env_err.retryable, false);
    }

    #[test]
    fn converts_anyhow_error_with_context() {
        let anyhow_err =
            anyhow::anyhow!("database connection failed").context("failed to fetch user");
        let env_err: Error = anyhow_err.into();

        assert_eq!(env_err.code, Code::Internal);
        assert!(env_err.message.contains("failed to fetch user"));
    }

    #[test]
    fn works_with_question_mark_operator() {
        fn anyhow_function() -> anyhow::Result<String> {
            Err(anyhow::anyhow!("test error"))
        }

        fn handler() -> Result<String, Error> {
            let result = anyhow_function()?;
            Ok(result)
        }

        let result = handler();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "test error");
    }
}
