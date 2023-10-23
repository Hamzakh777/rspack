use std::sync::Arc;

use derivative::Derivative;
use napi::{Env, JsFunction};
use napi_derive::napi;
use rspack_core::{
  to_identifier, BoxPlugin, CrossOriginLoading, FilenameFnCtx, LibraryAuxiliaryComment,
  LibraryName, LibraryOptions, OutputFilename, OutputOptions, TrustedTypes,
};
use rspack_error::internal_error;
use rspack_napi_shared::{
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  NapiResultExt, NAPI_ENV,
};
use serde::Deserialize;

use crate::RawOptionsApply;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawTrustedTypes {
  pub policy_name: Option<String>,
}

impl From<RawTrustedTypes> for TrustedTypes {
  fn from(value: RawTrustedTypes) -> Self {
    Self {
      policy_name: value.policy_name,
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawLibraryName {
  pub amd: Option<String>,
  pub commonjs: Option<String>,
  pub root: Option<Vec<String>>,
}

impl From<RawLibraryName> for LibraryName {
  fn from(value: RawLibraryName) -> Self {
    Self {
      amd: value.amd,
      commonjs: value.commonjs,
      root: value.root,
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawLibraryAuxiliaryComment {
  pub root: Option<String>,
  pub commonjs: Option<String>,
  pub commonjs2: Option<String>,
  pub amd: Option<String>,
}

impl From<RawLibraryAuxiliaryComment> for LibraryAuxiliaryComment {
  fn from(value: RawLibraryAuxiliaryComment) -> Self {
    Self {
      amd: value.amd,
      commonjs: value.commonjs,
      root: value.root,
      commonjs2: value.commonjs2,
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawLibraryOptions {
  pub name: Option<RawLibraryName>,
  pub export: Option<Vec<String>>,
  // webpack type
  pub library_type: String,
  pub umd_named_define: Option<bool>,
  pub auxiliary_comment: Option<RawLibraryAuxiliaryComment>,
}

impl From<RawLibraryOptions> for LibraryOptions {
  fn from(value: RawLibraryOptions) -> Self {
    Self {
      name: value.name.map(Into::into),
      export: value.export,
      library_type: value.library_type,
      umd_named_define: value.umd_named_define,
      auxiliary_comment: value.auxiliary_comment.map(Into::into),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawCrossOriginLoading {
  #[napi(ts_type = r#""bool" | "string""#)]
  pub r#type: String,
  pub string_payload: Option<String>,
  pub bool_payload: Option<bool>,
}

impl From<RawCrossOriginLoading> for CrossOriginLoading {
  fn from(value: RawCrossOriginLoading) -> Self {
    match value.r#type.as_str() {
      "string" => Self::Enable(
        value
          .string_payload
          .expect("should have a string_payload when RawCrossOriginLoading.type is \"string\""),
      ),
      "bool" => Self::Disable,
      _ => unreachable!(),
    }
  }
}

#[derive(Derivative, Deserialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawFilename {
  #[napi(ts_type = r#""string" | "function""#)]
  pub r#type: String,
  pub string_payload: Option<String>,
  #[derivative(Debug = "ignore")]
  #[serde(skip_deserializing)]
  pub fn_payload: Option<JsFunction>,
}

#[napi(object)]
pub struct RawFilenameFnCtx {
  pub hash: String,
}

impl TryFrom<RawFilename> for OutputFilename {
  type Error = rspack_error::Error;

  fn try_from(value: RawFilename) -> Result<Self, Self::Error> {
    match value.r#type.as_str() {
      "string" => {
        let s = value.string_payload.ok_or_else(|| {
          internal_error!("should have a string_payload when RawFilename.type is \"string\"")
        })?;
        Ok(OutputFilename::String(s))
      }
      "function" => {
        let func = value.fn_payload.ok_or_else(|| {
          internal_error!("should have a fn_payload when RawFilename.type is \"function\"")
        })?;
        let func: ThreadsafeFunction<RawFilenameFnCtx, String> =
          NAPI_ENV.with(|env| -> anyhow::Result<_> {
            let env = env.borrow().expect("Failed to get env with external");
            let func_use = rspack_binding_macros::js_fn_into_threadsafe_fn!(func, &Env::from(env));
            Ok(func_use)
          })?;
        let func = Arc::new(func);
        Ok(OutputFilename::Fn(Box::new(move |ctx: FilenameFnCtx| {
          let func = func.clone();
          Box::pin(async move {
            func
              .call(ctx.into(), ThreadsafeFunctionCallMode::NonBlocking)
              .into_rspack_result()?
              .await
              .map_err(|err| internal_error!("Failed to call rule.use function: {err}"))?
          })
        })))
      }
      _ => unreachable!(),
    }
  }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOutputOptions {
  pub path: String,
  pub clean: bool,
  pub public_path: String,
  pub asset_module_filename: String,
  pub wasm_loading: String,
  pub enabled_wasm_loading_types: Vec<String>,
  pub webassembly_module_filename: String,
  pub filename: RawFilename,
  pub chunk_filename: String,
  pub cross_origin_loading: RawCrossOriginLoading,
  pub css_filename: String,
  pub css_chunk_filename: String,
  pub hot_update_main_filename: String,
  pub hot_update_chunk_filename: String,
  pub hot_update_global: String,
  pub unique_name: String,
  pub chunk_loading_global: String,
  pub library: Option<RawLibraryOptions>,
  pub strict_module_error_handling: bool,
  pub enabled_library_types: Option<Vec<String>>,
  pub global_object: String,
  pub import_function_name: String,
  pub iife: bool,
  pub module: bool,
  pub chunk_loading: String,
  pub enabled_chunk_loading_types: Option<Vec<String>>,
  pub trusted_types: Option<RawTrustedTypes>,
  pub source_map_filename: String,
  pub hash_function: String,
  pub hash_digest: String,
  pub hash_digest_length: u32,
  pub hash_salt: Option<String>,
  pub async_chunks: bool,
  pub worker_chunk_loading: String,
  pub worker_wasm_loading: String,
  pub worker_public_path: String,
}

impl RawOptionsApply for RawOutputOptions {
  type Options = OutputOptions;
  fn apply(self, _: &mut Vec<BoxPlugin>) -> Result<OutputOptions, rspack_error::Error> {
    Ok(OutputOptions {
      path: self.path.into(),
      clean: self.clean,
      public_path: self.public_path.into(),
      asset_module_filename: self.asset_module_filename.into(),
      wasm_loading: self.wasm_loading.as_str().into(),
      webassembly_module_filename: self.webassembly_module_filename.into(),
      unique_name: self.unique_name,
      chunk_loading: self.chunk_loading.as_str().into(),
      chunk_loading_global: to_identifier(&self.chunk_loading_global),
      filename: self.filename.into(),
      chunk_filename: self.chunk_filename.into(),
      cross_origin_loading: self.cross_origin_loading.into(),
      css_filename: self.css_filename.into(),
      css_chunk_filename: self.css_chunk_filename.into(),
      hot_update_main_filename: self.hot_update_main_filename.into(),
      hot_update_chunk_filename: self.hot_update_chunk_filename.into(),
      hot_update_global: self.hot_update_global,
      library: self.library.map(Into::into),
      strict_module_error_handling: self.strict_module_error_handling,
      enabled_library_types: self.enabled_library_types,
      global_object: self.global_object,
      import_function_name: self.import_function_name,
      iife: self.iife,
      module: self.module,
      trusted_types: self.trusted_types.map(Into::into),
      source_map_filename: self.source_map_filename.into(),
      hash_function: self.hash_function.as_str().into(),
      hash_digest: self.hash_digest.as_str().into(),
      hash_digest_length: self.hash_digest_length as usize,
      hash_salt: self.hash_salt.into(),
      async_chunks: self.async_chunks,
      worker_chunk_loading: self.worker_chunk_loading.as_str().into(),
      worker_wasm_loading: self.worker_wasm_loading.as_str().into(),
      worker_public_path: self.worker_public_path,
    })
  }
}
