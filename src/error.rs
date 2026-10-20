pub trait IntoNvimErrorExt {
    fn into_nvim(self) -> nvim_oxi::Error;
}

pub trait IntoNvimResultExt<T> {
    fn into_nvim(self) -> nvim_oxi::Result<T>;
}

impl IntoNvimErrorExt for anyhow::Error {
    fn into_nvim(self) -> nvim_oxi::Error {
        log::error!("{self:?}");
        match self.downcast::<nvim_oxi::Error>() {
            Ok(err) => err,
            Err(err) => nvim_oxi::Error::Api(nvim_oxi::api::Error::Other(format!("{:#}", err))),
        }
    }
}

impl<T> IntoNvimResultExt<T> for anyhow::Result<T> {
    fn into_nvim(self) -> nvim_oxi::Result<T> {
        self.map_err(anyhow::Error::into_nvim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, bail};

    #[test]
    fn preserves_original_nvim_error() {
        let original = nvim_oxi::Error::Api(nvim_oxi::api::Error::Other("original error".into()));

        let result: anyhow::Result<()> = Err(original.clone().into());

        let converted = result.into_nvim().unwrap_err();

        assert!(matches!(
            converted,
            nvim_oxi::Error::Api(nvim_oxi::api::Error::Other(ref msg))
                if msg == "original error"
        ));
    }

    #[test]
    fn converts_anyhow_error() {
        fn fail() -> anyhow::Result<()> {
            bail!("something went wrong")
        }

        let converted = fail().into_nvim().unwrap_err();

        assert!(matches!(
            converted,
            nvim_oxi::Error::Api(nvim_oxi::api::Error::Other(ref msg))
                if msg == "something went wrong"
        ));
    }

    #[test]
    fn preserves_context_chain() {
        fn fail() -> anyhow::Result<()> {
            bail!("root cause")
        }

        let result = fail().context("first context").context("second context");

        let converted = result.into_nvim().unwrap_err();

        assert!(matches!(
            converted,
            nvim_oxi::Error::Api(nvim_oxi::api::Error::Other(ref msg))
                if msg == "second context: first context: root cause"
        ));
    }

    #[test]
    fn preserves_successful_result() {
        let result: anyhow::Result<i32> = Ok(42);

        assert_eq!(result.into_nvim().unwrap(), 42);
    }

    #[test]
    fn preserves_original_error_under_context() {
        let original = nvim_oxi::Error::Api(nvim_oxi::api::Error::Other("original error".into()));

        let result: anyhow::Result<()> = Err(original.into());

        let result = result
            .context("initialization failed")
            .context("plugin failed");

        let converted = result.into_nvim().unwrap_err();

        assert!(matches!(
            converted,
            nvim_oxi::Error::Api(nvim_oxi::api::Error::Other(ref msg))
                if msg == "original error"
        ));
    }
}
