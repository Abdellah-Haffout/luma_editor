Created At: 2026-10-16T23:20:12Z
Completed At: 2026-10-16T23:20:12Z
File Path: `file:///home/abdellah/Univis-project/luma_editor/src/main.rs`
Total Lines: 201
Total Bytes: 7800
Showing lines 60 to 140
The following code has been modified to include a line number before every line, in the format: <line_number>: <original_line>. Please note that any changes targeting the original code should remove the line number, colon, and leading space.
            true,
        )
        .with_shortcut("Space"),
    );

    registry.register(
        CommandDescriptor::new(
            "view.dive_in",
            "Focus / Dive-In",
            CommandCategory::View,
            "Expand the selected node into a dedicated fullscreen workspace",
            false,
        )
        .with_shortcut("Tab"),
    );

    // Register execution handlers for core commands
    dispatcher.0.register_handler("edit.undo", |ctx| {
        let _ = ctx.history.undo(ctx.graph);
        Ok(())
    });

    dispatcher.0.register_handler("edit.redo", |ctx| {
        let _ = ctx.history.redo(ctx.graph);
        Ok(())
    });

    dispatcher.0.register_handler("app.save_and_bake", |ctx| {
        let _ = std::fs::create_dir_all("scenes");

        // Demand-Pull all nodes through the open-closed DataflowEngine
        let pinned = std::collections::HashSet::new();
        for (&node_id, _) in ctx.graph.nodes() {
            let _ = ctx.dataflow.pull_node(ctx.graph, node_id, &pinned);
        }

        let file = luma_editor::scene::luma_file::LumaGraphFile::new(
            luma_editor::canvas::transform::CanvasTransform::default(),
            ctx.graph.clone(),
            ctx.payloads,
        );
        let res = luma_editor::scene::auto_bake::AutoBakePipeline::save_and_bake(
            "scenes/main.luma",
            "scenes/main.bsn",
            &file,
        );
        match res {
            Ok(bake) => {
                println!("[AutoBake] Successfully baked {} entities to {:?}", bake.entities_baked_count, bake.bsn_file_path);
                Ok(())
            }
            Err(e) => {
                eprintln!("[AutoBake Error] {}", e);
                Err(e)
            }
        }
    });

    // 3. Seed sample demo nodes onto the canvas (Pure Mathematical Pipeline: Number -> Add -> Multiply -> Sine)
    let palette = NodePalette::with_default_templates();
    let num_node = palette.spawn_node(
        "math.number",
        Vec2::new(-480.0, 40.0),
        &mut graph_res.0,
        &id_gen.0,
        &mut history_res.0,
    );
    let add_node = palette.spawn_node(
        "math.add",
        Vec2::new(-200.0, 40.0),
        &mut graph_res.0,
        &id_gen.0,
        &mut history_res.0,
    );
    let mul_node = palette.spawn_node(
        "math.multiply",
        Vec2::new(80.0, 40.0),
        &mut graph_res.0,
        &id_gen.0,
        &mut history_res.0,
    );
The above content does NOT show the entire file contents. If you need to view any lines of the file which were not shown to complete your task, call this tool again to view those lines.
