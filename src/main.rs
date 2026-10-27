use bevy::prelude::*;
use luma_editor::canvas::{EditorDispatcherResource, LumaEditorPlugin};
use luma_editor::commands::descriptor::{CommandCategory, CommandDescriptor};
use luma_editor::ui::LumaUiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Luma Editor - Node Graph Architecture".into(),
                resolution: (1280, 800).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(luma_editor::ui::EditorTheme::BG_CANVAS))
        .add_plugins(bevy_luma::prelude::LumaPlugin)
        .add_plugins(LumaEditorPlugin)
        .add_plugins(LumaUiPlugin)
        .add_systems(Startup, setup_editor)
        .run();
}

/// Initializes a pure, node-free editor workspace infrastructure.
/// Domain-specific nodes and demo graphs live in `examples/node_editor_showcase.rs`.
fn setup_editor(
    mut commands: Commands,
    mut dispatcher: ResMut<EditorDispatcherResource>,
) {
    // 1. Unified 2D camera for world canvas (pan/zoom/gizmos) and screen UI
    commands.spawn((
        Camera2d,
        luma_editor::canvas::plugin::EditorCamera,
        luma_editor::ui::EditorUiCamera,
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // 2. Register core infrastructure commands in the central command registry
    let registry = dispatcher.0.registry_mut();
    registry.register(
        CommandDescriptor::new(
            "app.save_and_bake",
            "Save and Bake Scene",
            CommandCategory::File,
            "Save .luma project source and bake .bsn scene for Bevy",
            false,
        )
        .with_shortcut("Ctrl+S"),
    );

    registry.register(
        CommandDescriptor::new(
            "graph.add_node",
            "Spawn Node",
            CommandCategory::Graph,
            "Spawn a new node on the canvas",
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
                println!(
                    "[AutoBake] Successfully baked {} entities to {:?}",
                    bake.entities_baked_count, bake.bsn_file_path
                );
                Ok(())
            }
            Err(e) => {
                eprintln!("[AutoBake Error] {}", e);
                Err(e)
            }
        }
    });
}
