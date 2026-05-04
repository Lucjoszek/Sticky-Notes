#![windows_subsystem = "windows"]

mod style;
use notes_app::{Note, NotesList};
use ply_engine::prelude::*;
use style::*;
use uuid::Uuid;

fn window_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: miniquad::conf::Conf {
            window_title: "Your Notes".to_owned(),
            window_width: 1280,
            window_height: 720,
            high_dpi: true,
            sample_count: 0,
            platform: miniquad::conf::Platform {
                webgl_version: miniquad::conf::WebGLVersion::WebGL2,
                ..Default::default()
            },
            ..Default::default()
        },
        draw_call_vertex_capacity: 100000,
        draw_call_index_capacity: 100000,
        ..Default::default()
    }
}

#[derive(Debug)]
struct AppState {
    notes_list: NotesList,
    show_dialog: bool,
    editing_note: Option<Uuid>,
}

#[macroquad::main(window_conf)]
async fn main() {
    // Default states
    let mut app_state = AppState {
        notes_list: NotesList::load(None).unwrap_or_else(|_| NotesList::new()),
        show_dialog: false,
        editing_note: None,
    };

    // Get files in bytes
    static FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/roboto.ttf");
    static STAR_EMPTY_BYTES: &[u8] = include_bytes!("../assets/icons/star-light-svgrepo-com.png");
    static STAR_FILLED_BYTES: &[u8] = include_bytes!("../assets/icons/star-svgrepo-com.png");

    // Load these files
    static DEFAULT_FONT: FontAsset = FontAsset::Bytes {
        file_name: "roboto.ttf",
        data: FONT_BYTES,
    };
    static STAR_EMPTY: GraphicAsset = GraphicAsset::Bytes {
        file_name: "star_empty.png",
        data: STAR_EMPTY_BYTES,
    };
    static STAR_FILLED: GraphicAsset = GraphicAsset::Bytes {
        file_name: "star_empty.png",
        data: STAR_FILLED_BYTES,
    };

    // Ply UI engine
    let mut ply = Ply::<()>::new(&DEFAULT_FONT).await;

    // Enable debug mode
    // ply.set_debug_mode(true);

    loop {
        clear_background(MacroquadColor::from_hex(BG_MAIN));

        // Dialog save button color change
        let mut can_be_saved = false;
        if app_state.show_dialog {
            can_be_saved = !(ply.get_text_value("new_title").trim().is_empty()
                || ply.get_text_value("new_content").trim().is_empty());
        }

        // Handle favorite
        if ply.is_just_pressed("favorite_note") {
            if let Some(id) = app_state.editing_note {
                if let Some(note) = app_state.notes_list.notes.iter_mut().find(|n| n.id == id) {
                    note.favorite = !note.favorite;
                    note.modification_date = chrono::Local::now().date_naive();
                }

                // Save changes
                app_state.notes_list.save(None).ok();
            }
        }

        // Handle remove
        if ply.is_just_pressed("remove_note") {
            if let Some(id) = app_state.editing_note {
                app_state.notes_list.remove(&id).unwrap();

                // Save changes to file
                app_state.notes_list.save(None).ok();
            }

            // Reset values
            ply.set_text_value("new_title", "");
            ply.set_text_value("new_content", "");
            ply.set_text_value("new_tags", "");

            // Reset states
            app_state.show_dialog = false;
            app_state.editing_note = None;
        }

        // Handle cancel
        if ply.is_just_pressed("cancel") {
            // Reset values
            ply.set_text_value("new_title", "");
            ply.set_text_value("new_content", "");
            ply.set_text_value("new_tags", "");

            // Reset states
            app_state.show_dialog = false;
            app_state.editing_note = None;
        }

        // Handle save
        if ply.is_just_pressed("save_note") {
            let title = ply.get_text_value("new_title");
            let content = ply.get_text_value("new_content");
            let tags_str = ply.get_text_value("new_tags");

            // Converts &str to Vec
            let tags: Vec<String> = tags_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if !(title.trim().is_empty() || content.trim().is_empty()) {
                match app_state.editing_note {
                    Some(id) => {
                        // Editing
                        if let Some(note) =
                            app_state.notes_list.notes.iter_mut().find(|n| n.id == id)
                        {
                            note.title = title.to_string();
                            note.content = content.to_string();
                            note.tags = tags; // Zapisz nowe tagi
                            note.modification_date = chrono::Local::now().date_naive();
                        }
                    }
                    None => {
                        // New
                        app_state.notes_list.add(Note::new(
                            title.to_string(),
                            content.to_string(),
                            tags, // Przekaż tagi do konstruktora
                            false,
                        ));
                    }
                }

                // Save changes to file
                app_state.notes_list.save(None).ok();

                // Reset values
                ply.set_text_value("new_title", "");
                ply.set_text_value("new_content", "");
                ply.set_text_value("new_tags", "");

                // Reset states
                app_state.show_dialog = false;
                app_state.editing_note = None;
            }
        }

        // Handle edit
        for (i, note) in app_state.notes_list.notes.iter().enumerate() {
            if ply.is_just_pressed(("note", i as u32)) {
                app_state.show_dialog = true;
                app_state.editing_note = Some(note.id);

                // Load values
                ply.set_text_value("new_title", &note.title);
                ply.set_text_value("new_content", &note.content);
                ply.set_text_value("new_tags", &note.tags.join(", "));
            }
        }

        let search_query = ply.get_text_value("search").trim().to_lowercase();

        let (favorited_indices, other_indices): (Vec<_>, Vec<_>) =
            (0..app_state.notes_list.notes.len())
                .filter(|&i| {
                    if search_query.is_empty() {
                        return true;
                    }
                    let note = &app_state.notes_list.notes[i];

                    // Szukamy w tytule
                    let title_match = note.title.to_lowercase().contains(&search_query);

                    // Szukamy w tagach
                    let tag_match = note
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&search_query));

                    title_match || tag_match
                })
                .partition(|&i| app_state.notes_list.notes[i].favorite);

        // let (favorited_indices, other_indices): (Vec<_>, Vec<_>) =
        //     (0..app_state.notes_list.notes.len())
        //         .partition(|&i| app_state.notes_list.notes[i].favorite);

        let mut ui = ply.begin();

        //
        // Root
        //
        ui.element()
            .width(grow!())
            .height(grow!())
            .layout(|l| {
                l.direction(TopToBottom)
                    .align(CenterX, Top)
                    .padding(SPACE_4)
                    .gap(SPACE_5)
            })
            .children(|ui| {
                //
                // Header
                //
                ui.element()
                    .width(grow!())
                    .height(fixed!(36.0))
                    .layout(|l| l.gap(SPACE_4))
                    .children(|ui| {
                        //
                        // Spacer
                        //
                        ui.element().width(grow!()).height(grow!()).empty();

                        //
                        // Search bar
                        //
                        ui.element()
                            .id("search")
                            .width(fixed!(400.0))
                            .height(grow!())
                            .background_color(BG_SURFACE)
                            .text_input(|t| {
                                t.placeholder("Search through notes... (title or tag)")
                                    .font_size(TEXT_SM)
                                    .text_color(TEXT_PRIMARY)
                                    .placeholder_color(TEXT_SECONDARY)
                            })
                            .empty();

                        //
                        // Buttons
                        //
                        ui.element()
                            .width(grow!())
                            .height(grow!())
                            .layout(|l| l.gap(SPACE_4))
                            .children(|ui| {
                                //
                                // New note button
                                //
                                if ui.just_pressed() {
                                    app_state.show_dialog = true;
                                }

                                ui.element()
                                    .width(fixed!(36.0))
                                    .height(grow!())
                                    .background_color(BG_SURFACE)
                                    .layout(|l| l.align(CenterX, CenterY))
                                    .children(|ui| {
                                        ui.text("+", |t| t.font_size(TEXT_SM).color(TEXT_PRIMARY));
                                    });
                            });
                    });

                //
                // Main
                //
                ui.element()
                    .width(grow!())
                    .height(grow!())
                    .overflow(|o| o.scroll_y())
                    .layout(|l| l.direction(TopToBottom).gap(SPACE_5))
                    .children(|ui| {
                        //
                        // Favorited notes
                        //
                        if !favorited_indices.is_empty() {
                            //
                            // Section title
                            //
                            ui.element()
                                .width(grow!())
                                .border(|b| b.bottom(2).color(BG_SURFACE))
                                .layout(|l| l.padding((0, 0, SPACE_1, 0)))
                                .children(|ui| {
                                    ui.text("Favorited", |t| {
                                        t.font_size(TEXT_XL).color(TEXT_PRIMARY)
                                    });
                                });

                            //
                            // Notes
                            //
                            ui.element()
                                .width(grow!())
                                .height(grow!())
                                .layout(|l| l.gap(SPACE_4).wrap().wrap_gap(SPACE_4))
                                .children(|ui| {
                                    for &i in &favorited_indices {
                                        let note = &app_state.notes_list.notes[i];

                                        //
                                        // Notes
                                        //
                                        ui.element()
                                            .id(("note", i as u32))
                                            .corner_radius(12.0)
                                            .width(percent!(0.33))
                                            .height(fixed!(400.0))
                                            .background_color(BG_CARD)
                                            .layout(|l| {
                                                l.padding(SPACE_4)
                                                    .direction(TopToBottom)
                                                    .gap(SPACE_3)
                                            })
                                            .children(|ui| {
                                                //
                                                // Note title
                                                //
                                                ui.text(&note.title, |t| {
                                                    t.font_size(TEXT_LG)
                                                        .color(TEXT_PRIMARY)
                                                        .wrap_mode(WrapMode::Words)
                                                });

                                                //
                                                // Note content
                                                //
                                                ui.element()
                                                    .width(grow!())
                                                    .height(grow!())
                                                    .children(|ui| {
                                                        ui.text(&note.content, |t| {
                                                            t.font_size(TEXT_SM)
                                                                .color(TEXT_PRIMARY)
                                                                .wrap_mode(WrapMode::Words)
                                                        });
                                                    });

                                                //
                                                // Note tags
                                                //
                                                if !note.tags.is_empty() {
                                                    ui.text(
                                                        &format!("Tags: {}", note.tags.join(", ")),
                                                        |t| {
                                                            t.font_size(TEXT_SM)
                                                                .color(TEXT_PRIMARY)
                                                                .wrap_mode(WrapMode::Words)
                                                        },
                                                    );
                                                }

                                                //
                                                // Note last modification date
                                                //
                                                ui.text(
                                                    &format!(
                                                        "Modified: {}",
                                                        note.modification_date
                                                    ),
                                                    |t| {
                                                        t.font_size(TEXT_XS)
                                                            .color(TEXT_SECONDARY)
                                                            .alignment(Right)
                                                    },
                                                );
                                            });
                                    }
                                });
                        }

                        //
                        // Others notes
                        //
                        if !other_indices.is_empty() {
                            //
                            // Section title
                            //
                            if !favorited_indices.is_empty() {
                                ui.element()
                                    .width(grow!())
                                    .border(|b| b.bottom(2).color(BG_SURFACE))
                                    .layout(|l| l.padding((0, 0, SPACE_1, 0)))
                                    .children(|ui| {
                                        ui.text("Others", |t| {
                                            t.font_size(TEXT_XL).color(TEXT_PRIMARY)
                                        });
                                    });
                            }

                            //
                            // Notes
                            //
                            ui.element()
                                .width(grow!())
                                .height(grow!())
                                .layout(|l| l.gap(SPACE_4).wrap().wrap_gap(SPACE_4))
                                .children(|ui| {
                                    for &i in &other_indices {
                                        let note = &app_state.notes_list.notes[i];

                                        //
                                        // Note
                                        //
                                        ui.element()
                                            .id(("note", i as u32))
                                            .corner_radius(12.0)
                                            .width(percent!(0.33))
                                            .height(fixed!(400.0))
                                            .background_color(BG_CARD)
                                            .layout(|l| {
                                                l.padding(SPACE_4)
                                                    .direction(TopToBottom)
                                                    .gap(SPACE_3)
                                            })
                                            .children(|ui| {
                                                //
                                                // Note title
                                                //
                                                ui.text(&note.title, |t| {
                                                    t.font_size(TEXT_LG)
                                                        .color(TEXT_PRIMARY)
                                                        .wrap_mode(WrapMode::Words)
                                                });

                                                //
                                                // Note content
                                                //
                                                ui.element()
                                                    .width(grow!())
                                                    .height(grow!())
                                                    .children(|ui| {
                                                        ui.text(&note.content, |t| {
                                                            t.font_size(TEXT_SM)
                                                                .color(TEXT_PRIMARY)
                                                                .wrap_mode(WrapMode::Words)
                                                        });
                                                    });

                                                //
                                                // Note tags
                                                //
                                                if !note.tags.is_empty() {
                                                    ui.text(
                                                        &format!("Tags: {}", note.tags.join(", ")),
                                                        |t| {
                                                            t.font_size(TEXT_SM)
                                                                .color(TEXT_PRIMARY)
                                                                .wrap_mode(WrapMode::Words)
                                                        },
                                                    );
                                                }

                                                //
                                                // Note last modification date
                                                //
                                                ui.text(
                                                    &format!(
                                                        "Modified: {}",
                                                        note.modification_date
                                                    ),
                                                    |t| {
                                                        t.font_size(TEXT_XS)
                                                            .color(TEXT_SECONDARY)
                                                            .alignment(Right)
                                                    },
                                                );
                                            });
                                    }
                                });
                        }

                        //
                        // No notes
                        //
                        if app_state.notes_list.notes.is_empty() {
                            ui.element()
                                .width(grow!())
                                .height(grow!())
                                .layout(|l| l.align(CenterX, CenterY).direction(TopToBottom))
                                .children(|ui| {
                                    ui.text("Nothing to show", |t| {
                                        t.font_size(TEXT_XL).color(TEXT_PRIMARY)
                                    });
                                });
                        }
                    });

                //
                // Editing dialog
                //
                if app_state.show_dialog {
                    //
                    // Overlay
                    //
                    ui.element()
                        .width(grow!())
                        .height(grow!())
                        .background_color(MacroquadColor::from_hex(BG_MAIN).with_alpha(0.67))
                        .floating(|f| f.attach_root())
                        .empty();

                    //
                    // Dialog
                    //
                    ui.element()
                        .width(fixed!(500.0))
                        .height(fixed!(435.0))
                        .background_color(BG_SURFACE)
                        .floating(|f| {
                            f.attach_parent()
                                .clip_by_parent()
                                .anchor((CenterX, CenterY), (CenterX, CenterY))
                        })
                        .layout(|l| l.direction(TopToBottom).gap(SPACE_3).padding(SPACE_4))
                        .children(|ui| {
                            //
                            // Header
                            //
                            ui.element()
                                .width(grow!())
                                .height(fixed!(32.0))
                                .children(|ui| {
                                    //
                                    // Left side
                                    //
                                    ui.element()
                                        .width(grow!())
                                        .height(grow!())
                                        .layout(|l| l.align(Left, CenterY).gap(SPACE_3))
                                        .children(|ui| {
                                            //
                                            // Dialog title
                                            //
                                            ui.text(
                                                if app_state.editing_note.is_some() {
                                                    "Edit Note"
                                                } else {
                                                    "New Note"
                                                },
                                                |t| t.font_size(TEXT_XL).color(TEXT_PRIMARY),
                                            );

                                            //
                                            // Favorite button
                                            //
                                            if app_state.editing_note.is_some() {
                                                ui.element()
                                                    .id("favorite_note")
                                                    .width(fixed!(20.0))
                                                    .height(fixed!(20.0))
                                                    .background_color(WHITE)
                                                    .layout(|l| l.align(CenterX, CenterY))
                                                    .image(
                                                        if let Some(id) = app_state.editing_note {
                                                            if let Some(note) = app_state
                                                                .notes_list
                                                                .notes
                                                                .iter()
                                                                .find(|n| n.id == id)
                                                            {
                                                                if note.favorite {
                                                                    &STAR_FILLED
                                                                } else {
                                                                    &STAR_EMPTY
                                                                }
                                                            } else {
                                                                &STAR_EMPTY
                                                            }
                                                        } else {
                                                            &STAR_EMPTY
                                                        },
                                                    )
                                                    .empty();
                                            }
                                        });

                                    //
                                    // Right side
                                    //
                                    ui.element()
                                        .width(grow!())
                                        .height(grow!())
                                        .layout(|l| l.align(Right, CenterY))
                                        .children(|ui| {
                                            //
                                            // Cancel/Close button
                                            //
                                            ui.element()
                                                .id("cancel")
                                                .width(fixed!(32.0))
                                                .height(grow!())
                                                .layout(|l| l.align(CenterX, CenterY))
                                                .background_color(BG_MAIN)
                                                .children(|ui| {
                                                    ui.text("x", |t| {
                                                        t.font_size(TEXT_SM).color(TEXT_PRIMARY)
                                                    });
                                                });
                                        });
                                });

                            //
                            // Title label
                            //
                            ui.element()
                                .width(grow!())
                                .layout(|l| l.direction(TopToBottom).gap(SPACE_3))
                                .children(|ui| {
                                    ui.text("Title", |t| t.font_size(TEXT_LG).color(TEXT_PRIMARY));

                                    //
                                    // Title input
                                    //
                                    ui.element()
                                        .id("new_title")
                                        .width(grow!())
                                        .height(fixed!(36.0))
                                        .border(|b| b.bottom(1).color(BORDER))
                                        .text_input(|t| {
                                            t.placeholder("Lorem ipsum")
                                                .font_size(TEXT_SM)
                                                .text_color(TEXT_PRIMARY)
                                        })
                                        .empty();
                                });

                            //
                            // Content label
                            //
                            ui.element()
                                .width(grow!())
                                .layout(|l| l.direction(TopToBottom).gap(SPACE_3))
                                .children(|ui| {
                                    ui.text("Content", |t| {
                                        t.font_size(TEXT_LG).color(TEXT_PRIMARY)
                                    });

                                    //
                                    // Content input
                                    //
                                    ui.element()
                                        .id("new_content")
                                        .width(grow!())
                                        .height(fixed!(120.0))
                                        .border(|b| b.bottom(1).color(BORDER))
                                        .text_input(|t| {
                                            t.placeholder("Lorem ipsum dolor sit amet...")
                                                .font_size(TEXT_SM)
                                                .text_color(TEXT_PRIMARY)
                                                .multiline()
                                                .scrollbar(|s| s.width(4.0).thumb_color(0xC8C8C8))
                                        })
                                        .empty();
                                });

                            //
                            // Tags label
                            //
                            ui.element()
                                .width(grow!())
                                .layout(|l| l.direction(TopToBottom).gap(SPACE_3))
                                .border(|b| b.bottom(1).color(BORDER))
                                .children(|ui| {
                                    ui.text("Tags", |t| t.font_size(TEXT_LG).color(TEXT_PRIMARY));

                                    //
                                    // Tags input
                                    //
                                    ui.element()
                                        .id("new_tags")
                                        .width(grow!())
                                        .height(fixed!(36.0))
                                        .text_input(|t| {
                                            t.placeholder("tag1, tag2, ...")
                                                .font_size(TEXT_SM)
                                                .text_color(TEXT_PRIMARY)
                                        })
                                        .empty();
                                });

                            //
                            // Bottom buttons
                            //
                            ui.element()
                                .width(grow!())
                                .layout(|l| l.align(Right, Bottom).gap(SPACE_3))
                                .children(|ui| {
                                    //
                                    // Save button
                                    //
                                    ui.element()
                                        .id("save_note")
                                        .width(fixed!(64.0))
                                        .height(fixed!(32.0))
                                        .background_color(if can_be_saved {
                                            ACCENT
                                        } else {
                                            BG_SURFACE
                                        })
                                        .layout(|l| l.align(CenterX, CenterY))
                                        .children(|ui| {
                                            ui.text("Save", |t| {
                                                t.font_size(TEXT_SM).color(TEXT_PRIMARY)
                                            });
                                        });

                                    //
                                    // Remove button
                                    //
                                    if app_state.editing_note.is_some() {
                                        ui.element()
                                            .id("remove_note")
                                            .width(fixed!(64.0))
                                            .height(fixed!(32.0))
                                            .background_color(ACCENT_2)
                                            .layout(|l| l.align(CenterX, CenterY).padding(SPACE_4))
                                            .children(|ui| {
                                                ui.text("Remove", |t| {
                                                    t.font_size(TEXT_SM).color(TEXT_PRIMARY)
                                                });
                                            });
                                    }
                                });
                        });
                }
            });

        ui.show(|_| {}).await;

        next_frame().await;
    }
}
