use std::borrow::Cow;

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  ChunkGraph, Compilation, GenerateContext, Generator, Module, ModuleGraph, NormalModule,
  RuntimeSpec, SourceType, rspack_sources::BoxSource,
};
use rspack_error::Result;
use rspack_hash::RspackHashDigest;
use rspack_plugin_asset::{AssetGenerator, AssetParserData};
use rustc_hash::FxHashSet;

#[cacheable]
#[derive(Debug, Clone)]
pub(crate) struct RslibAssetGenerator(pub AssetGenerator);

#[cacheable_dyn]
#[async_trait::async_trait]
impl Generator for RslibAssetGenerator {
  fn source_types(&self, module: &dyn Module, module_graph: &ModuleGraph) -> &[SourceType] {
    let mut source_types = FxHashSet::default();
    let module_id = module.identifier();
    for connection in module_graph.get_incoming_connections(&module_id) {
      if let Some(module) = connection
        .original_module_identifier
        .and_then(|id| module_graph.module_by_identifier(&id))
      {
        let module_type = module.module_type();
        source_types.insert(SourceType::from(module_type));
      }
    }

    if source_types.is_empty()
      && module
        .as_normal_module()
        .and_then(|module| module.parser_data())
        .and_then(|data| data.downcast_ref::<AssetParserData>())
        .is_some_and(|data| {
          !data.parsed_asset_config.is_inline() && !data.parsed_asset_config.is_source()
        })
    {
      return &[SourceType::JavaScript, SourceType::Asset];
    }

    self.0.source_types(module, module_graph)
  }

  fn size(&self, module: &dyn Module, source_type: Option<&SourceType>) -> f64 {
    self.0.size(module, source_type)
  }

  async fn generate(
    &self,
    source: &BoxSource,
    module: &dyn Module,
    generate_context: &mut GenerateContext,
  ) -> Result<BoxSource> {
    self.0.generate(source, module, generate_context).await
  }

  fn get_concatenation_bailout_reason(
    &self,
    module: &dyn Module,
    mg: &ModuleGraph,
    cg: &ChunkGraph,
  ) -> Option<Cow<'static, str>> {
    self.0.get_concatenation_bailout_reason(module, mg, cg)
  }

  async fn get_runtime_hash(
    &self,
    module: &NormalModule,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    self.0.get_runtime_hash(module, compilation, runtime).await
  }
}
