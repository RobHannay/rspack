use futures::Future;
use rspack_napi::napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use rspack_napi::napi::{bindgen_prelude::*, Env, JsFunction, NapiRaw, Result};

pub fn callbackify<R, F>(env: Env, f: JsFunction, fut: F) -> Result<()>
where
  R: 'static + ToNapiValue,
  F: 'static + Send + Future<Output = Result<R>>,
{
  let tsfn = unsafe {
    ThreadsafeFunction::<R, Unknown, /* CalleeHandled = */ true>::from_napi_value(
      env.raw(),
      f.raw(),
    )
  }?;
  napi::bindgen_prelude::spawn(async move {
    let res = fut.await;
    tsfn.call(res, ThreadsafeFunctionCallMode::NonBlocking);
  });
  Ok(())
}
