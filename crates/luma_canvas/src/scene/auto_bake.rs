use crate::scene::bsn_export::BsnCompiler;
use crate::scene::luma_file::LumaGraphFile;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AutoBakeResult {
    pub luma_file_path: PathBuf,
    pub bsn_file_path: PathBuf,
    pub entities_baked_count: usize,
}

pub struct AutoBakePipeline;

impl AutoBakePipeline {
    /// Saves the Luma Editor project source file (.luma) and simultaneously
    /// compiles/bakes the game-ready Bevy scene (.bsn) file.
    pub fn save_and_bake(
        luma_path: impl AsRef<Path>,
        bsn_path: impl AsRef<Path>,
        graph_file: &LumaGraphFile,
    ) -> Result<AutoBakeResult, String> {
        let luma_path_buf = luma_path.as_ref().to_path_buf();
        let bsn_path_buf = bsn_path.as_ref().to_path_buf();

        // 1. Save .luma project source file (Editor Source of Truth)
        graph_file.save_to_file(&luma_path_buf)?;

        // 2. Compile to .bsn scene file (Zero-runtime game export)
        let payloads = graph_file.to_domain_store();
        BsnCompiler::compile_to_file(&bsn_path_buf, &graph_file.graph, &payloads)?;

        let entities_baked_count = graph_file
            .graph
            .nodes()
            .values()
            .filter(|n| n.kind == "EntityAssembler")
            .count();

        Ok(AutoBakeResult {
            luma_file_path: luma_path_buf,
            bsn_file_path: bsn_path_buf,
            entities_baked_count,
        })
    }
}
