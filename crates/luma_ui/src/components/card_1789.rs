            let num_rows = layout.input_pins.len().max(layout.output_pins.len()).max(
                if layout.body_style == NodeBodyStyle::Parameters { 1 } else { 0 }
            );

            for row_idx in 0..num_rows {
                body.spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(24.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    position_type: PositionType::Relative,
                    ..default()
                }).with_children(|row| {
                    // 1. Left: Input Pin Dot + Label
                    if let Some(in_pin) = layout.input_pins.get(row_idx) {
                        let is_connected = layout.is_pin_connected(in_pin.id);
                        let color = get_blender_pin_color(&in_pin.data_type.name);

                        row.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(6.0),
                            margin: UiRect::left(Val::Px(-9.0)),
                            ..default()
                        }).with_children(|in_group| {
                            // Socket dot
                            in_group.spawn((
                                NodeCardSocketPin {
                                    node_id: layout.node_id,
                                    pin_id: Some(in_pin.id),
                                },
                                Node {
                                    width: Val::Px(10.0),
                                    height: Val::Px(10.0),
                                    border: UiRect::all(Val::Px(if is_connected { 1.0 } else { 2.0 })),
                                    border_radius: BorderRadius::all(Val::Px(5.0)),
                                    ..default()
                                },
                                BackgroundColor(if is_connected { color } else { Color::srgb(0.133, 0.141, 0.157) }),
                                BorderColor::all(color),
                            ));

                            // Label
                            let text_val = layout.input_display_texts.get(row_idx).cloned().unwrap_or_else(|| in_pin.name.clone());
                            in_group.spawn((
                                NodeInputValLabel {
                                    node_id: layout.node_id,
                                    slot_idx: row_idx,
                                },
                                Text::new(text_val),
                                TextFont { font_size: FontSize::Px(10.5), ..default() },
                                TextColor(Color::srgb(0.85, 0.88, 0.92)),
                            ));
                        });
                    } else {
                        row.spawn(Node::default());
                    }

                    // 2. Center: Parameter Stepper [-] [Value] [+]
                    if layout.body_style == NodeBodyStyle::Parameters && row_idx == 0 {