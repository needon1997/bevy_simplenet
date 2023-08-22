//local shortcuts

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[cfg(not(wasm))]
mod envmod
{
    /// Default IO runtime (tokio).
    /// If you access this via `::default()`, you will get a handle to a statically-initialized tokio runtime.
    #[derive(Clone, Debug)]
    pub struct DefaultIOHandle(pub tokio::runtime::Handle);

    impl Default for DefaultIOHandle
    {
        fn default() -> DefaultIOHandle
        {
            static RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();

            let runtime = RUNTIME.get_or_init(
                    || {
                        tokio::runtime::Runtime::new().expect("unable to get default tokio runtime")
                    }
                );
            DefaultIOHandle(runtime.handle().clone())
        }
    }

    impl From<DefaultIOHandle> for tokio::runtime::Handle {
        fn from(runtime: DefaultIOHandle) -> Self {
            runtime.0
        }
    }

    impl From<&DefaultIOHandle> for tokio::runtime::Handle {
        fn from(runtime: &DefaultIOHandle) -> Self {
            runtime.0.clone()
        }
    }

    impl From<tokio::runtime::Handle> for DefaultIOHandle {
        fn from(handle: tokio::runtime::Handle) -> Self {
            Self(handle)
        }
    }

    impl From<&tokio::runtime::Handle> for DefaultIOHandle {
        fn from(handle: &tokio::runtime::Handle) -> Self {
            Self(handle.clone())
        }
    }

    /// Default CPU runtime (unspecified)
    #[derive(Default)]
    pub struct DefaultCPURuntime;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[cfg(wasm)]
mod envmod
{
    use crate::*;

    /// Default CPU runtime (unspecified)
    #[derive(Clone, Debug, Default)]
    pub struct DefaultIOHandle;

    /// Default CPU runtime (unspecified)
    #[derive(Clone, Debug, Default)]
    pub struct DefaultCPURuntime;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub type DefaultIOHandle  = envmod::DefaultIOHandle;
pub type DefaultCPURuntime = envmod::DefaultCPURuntime;

//-------------------------------------------------------------------------------------------------------------------
