//!  There are methods whose verb is `ChunkGraphModule`
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

use dashmap::mapref::one::{Ref as DashMapRef, RefMut as DashMapRefMut};
use rspack_identifier::{Identifier, IdentifierHasher};
use rspack_util::ext::DynHash;
use rustc_hash::FxHashSet as HashSet;

use crate::{
  AsyncDependenciesBlockId, BoxModule, ChunkByUkey, ChunkGroup, ChunkGroupByUkey, ChunkGroupUkey,
  ChunkUkey, ExportsHash, ModuleIdentifier, RuntimeGlobals, RuntimeSpec, RuntimeSpecMap,
  RuntimeSpecSet,
};
use crate::{ChunkGraph, ModuleGraph};

#[derive(Debug, Clone, Default)]
pub struct ChunkGraphModule {
  pub id: Option<String>,
  pub(crate) entry_in_chunks: HashSet<ChunkUkey>,
  pub chunks: HashSet<ChunkUkey>,
  pub(crate) runtime_requirements: Option<RuntimeSpecMap<RuntimeGlobals>>,
  pub(crate) runtime_in_chunks: HashSet<ChunkUkey>,
  // pub(crate) hashes: Option<RuntimeSpecMap<u64>>,
}

impl ChunkGraphModule {
  pub fn new() -> Self {
    Self {
      id: None,
      entry_in_chunks: Default::default(),
      chunks: Default::default(),
      runtime_requirements: None,
      runtime_in_chunks: Default::default(),
      // hashes: None,
    }
  }
}

impl ChunkGraph {
  pub fn is_module_in_chunk(
    &self,
    module_identifier: &ModuleIdentifier,
    chunk_ukey: ChunkUkey,
  ) -> bool {
    let chunk_graph_chunk = self.get_chunk_graph_chunk(&chunk_ukey);
    chunk_graph_chunk.modules.contains(module_identifier)
  }

  pub(crate) fn get_chunk_graph_module_mut(
    &self,
    module_identifier: ModuleIdentifier,
  ) -> DashMapRefMut<'_, Identifier, ChunkGraphModule, BuildHasherDefault<IdentifierHasher>> {
    self
      .chunk_graph_module_by_module_identifier
      .entry(module_identifier)
      .or_default()
  }

  pub(crate) fn get_chunk_graph_module(
    &self,
    module_identifier: ModuleIdentifier,
  ) -> DashMapRef<'_, Identifier, ChunkGraphModule, BuildHasherDefault<IdentifierHasher>> {
    if !self
      .chunk_graph_module_by_module_identifier
      .contains_key(&module_identifier)
    {
      self
        .chunk_graph_module_by_module_identifier
        .entry(module_identifier)
        .or_default();
    }
    self
      .chunk_graph_module_by_module_identifier
      .get(&module_identifier)
      .expect("cgm must have been added")
  }

  pub fn get_module_chunks(&self, module_identifier: ModuleIdentifier) -> HashSet<ChunkUkey> {
    let chunk_graph_module = self.get_chunk_graph_module(module_identifier);
    chunk_graph_module.chunks.clone()
  }

  pub fn get_number_of_module_chunks(&self, module_identifier: ModuleIdentifier) -> usize {
    let cgm = self.get_chunk_graph_module(module_identifier);
    cgm.chunks.len()
  }

  pub fn add_module_runtime_requirements(
    &mut self,
    module_identifier: ModuleIdentifier,
    runtime: &RuntimeSpec,
    runtime_requirements: RuntimeGlobals,
  ) {
    let mut cgm = self.get_chunk_graph_module_mut(module_identifier);

    if let Some(runtime_requirements_map) = &mut cgm.runtime_requirements {
      if let Some(value) = runtime_requirements_map.get_mut(runtime) {
        value.insert(runtime_requirements);
      } else {
        runtime_requirements_map.set(runtime.clone(), runtime_requirements);
      }
    } else {
      let mut runtime_requirements_map = RuntimeSpecMap::default();
      runtime_requirements_map.set(runtime.clone(), runtime_requirements);
      cgm.runtime_requirements = Some(runtime_requirements_map);
    }
  }

  pub fn get_module_runtime_requirements(
    &self,
    module_identifier: ModuleIdentifier,
    runtime: &RuntimeSpec,
  ) -> Option<RuntimeGlobals> {
    let cgm = self.get_chunk_graph_module(module_identifier);
    if let Some(runtime_requirements) = &cgm.runtime_requirements {
      if let Some(runtime_requirements) = runtime_requirements.get(runtime) {
        return Some(*runtime_requirements);
      }
    }
    None
  }

  pub fn get_module_runtimes(
    &self,
    module_identifier: ModuleIdentifier,
    chunk_by_ukey: &ChunkByUkey,
  ) -> RuntimeSpecSet {
    let cgm = self.get_chunk_graph_module(module_identifier);
    let mut runtimes = RuntimeSpecSet::default();
    for chunk_ukey in cgm.chunks.iter() {
      let chunk = chunk_by_ukey.get(chunk_ukey).expect("Chunk should exist");
      runtimes.set(chunk.runtime.clone());
    }
    runtimes
  }

  pub fn get_module_id(&self, module_identifier: ModuleIdentifier) -> Option<String> {
    let cgm = self.get_chunk_graph_module(module_identifier);
    cgm.id.to_owned()
  }

  pub fn set_module_id(&mut self, module_identifier: ModuleIdentifier, id: String) {
    let mut cgm = self.get_chunk_graph_module_mut(module_identifier);
    cgm.id = Some(id);
  }

  /// Notice, you should only call this function with a ModuleIdentifier that's imported dynamically or
  /// is entry module.
  pub fn get_block_chunk_group<'a>(
    &self,
    block: &AsyncDependenciesBlockId,
    chunk_group_by_ukey: &'a ChunkGroupByUkey,
  ) -> Option<&'a ChunkGroup> {
    self
      .block_to_chunk_group_ukey
      .get(block)
      .and_then(|ukey| chunk_group_by_ukey.get(ukey))
  }

  pub fn connect_block_and_chunk_group(
    &mut self,
    block: AsyncDependenciesBlockId,
    chunk_group: ChunkGroupUkey,
  ) {
    self.block_to_chunk_group_ukey.insert(block, chunk_group);
  }

  pub fn get_module_graph_hash(
    &self,
    module: &BoxModule,
    module_graph: &ModuleGraph,
    with_connections: bool,
  ) -> String {
    let mut hasher = DefaultHasher::new();
    let mut connection_hash_cache: HashMap<Identifier, u64> = HashMap::new();

    fn process_module_graph_module(
      module: &BoxModule,
      module_graph: &ModuleGraph,
      strict: bool,
    ) -> u64 {
      let mut hasher = DefaultHasher::new();
      module.identifier().dyn_hash(&mut hasher);
      module.source_types().dyn_hash(&mut hasher);
      module_graph
        .is_async(&module.identifier())
        .dyn_hash(&mut hasher);

      module_graph
        .get_exports_info(&module.identifier())
        .export_info_hash(&mut hasher, module_graph);

      if let Some(mgm) = module_graph.module_graph_module_by_identifier(&module.identifier()) {
        let export_type = mgm.get_exports_type(strict);
        export_type.dyn_hash(&mut hasher);
      }

      hasher.finish()
    }

    // hash module build_info
    module_graph
      .get_module_hash(&module.identifier())
      .dyn_hash(&mut hasher);
    // hash module graph module
    process_module_graph_module(module, module_graph, false).dyn_hash(&mut hasher);

    let strict: bool = module_graph
      .module_graph_module_by_identifier(&module.identifier())
      .unwrap_or_else(|| {
        panic!(
          "Module({}) should be added before using",
          module.identifier()
        )
      })
      .get_strict_harmony_module();

    if with_connections {
      let mut connections = module_graph
        .get_outgoing_connections(module)
        .into_iter()
        .collect::<Vec<_>>();

      connections.sort_by(|a, b| a.module_identifier.cmp(&b.module_identifier));

      // hash connection module graph modules
      for connection in connections {
        if let Some(connection_hash) = connection_hash_cache.get(&connection.module_identifier) {
          connection_hash.dyn_hash(&mut hasher)
        } else {
          let connection_hash = process_module_graph_module(
            module_graph
              .module_by_identifier(&connection.module_identifier)
              .unwrap_or_else(|| {
                panic!(
                  "Module({}) should be added before using",
                  connection.module_identifier
                )
              }),
            module_graph,
            strict,
          );
          connection_hash.dyn_hash(&mut hasher);
          connection_hash_cache.insert(connection.module_identifier, connection_hash);
        }
      }
    }

    format!("{:016x}", hasher.finish())
  }
}
