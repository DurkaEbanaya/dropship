use eframe::egui;

use crate::overwatch::ServerSelection;
use crate::{api, visuals};

use crate::assets;

/// draw the stars here
pub fn server_list_indicators(
    ui: &mut egui::Ui,
    servers: &[api::KnownServer],
    desired_blocked_servers: &ServerSelection,
    actually_blocked_servers: ServerSelection,
    //
    theme: visuals::Theme,
) {
    servers.into_iter().enumerate().for_each(|(i, server)| {
        let blocked = actually_blocked_servers.has(server);
        let pending = desired_blocked_servers.has(server) != actually_blocked_servers.has(server);

        let color = if pending {
            crate::visuals::color_secondary_faded(i)
        } else {
            if blocked {
                // ui.visuals().weak_text_color()
                visuals::from_theme_alpha(theme, 9)
            } else {
                crate::visuals::color_active(i)
            }
        };

        let text = if pending {
            format!("{} (waiting for game to close..)", server.token)
        } else {
            if blocked {
                format!("{} (blocked)", server.token)
            } else {
                server.token.clone()
            }
        };

        ui.add(
            egui::Image::new(assets::ICON_STAR)
                .fit_to_exact_size(egui::vec2(16., 16.))
                .tint(color),
        )
        .on_hover_text_at_pointer(text);
    });
}

/// draw a writable selection
pub fn server_list(
    ui: &mut egui::Ui,
    servers: &[api::KnownServer],
    desired_blocked_servers: &mut ServerSelection,
    blocked_servers: ServerSelection,
    //
    pings: &crate::ping::Measurements,
    refresh_requests: &mut Vec<String>,
    //
    theme: visuals::Theme,
) -> bool {
    //
    let mut selection_changed = false;

    servers.iter().enumerate().for_each(|(i, server)| {
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // A separate hit target keeps ping refreshes from changing firewall selection.
                let refresh = egui::Button::image(
                    egui::Image::new(assets::ICON_REFRESH)
                        .tint(ui.visuals().text_color())
                        .fit_to_exact_size(egui::vec2(14.0, 14.0)),
                )
                .min_size(egui::vec2(24.0, 32.0));
                if ui
                    .add_enabled(!pings.is_measuring(&server.ping), refresh)
                    .on_hover_text(if pings.is_measuring(&server.ping) {
                        "Ping measurement in progress"
                    } else {
                        "Refresh this server's ping"
                    })
                    .clicked()
                {
                    refresh_requests.push(server.ping.clone());
                }
                if let Some(button) = server_list_item(
                    ui,
                    server,
                    desired_blocked_servers.has(server),
                    blocked_servers.has(server),
                    i,
                    pings,
                    //
                    theme,
                ) {
                    if button.hovered() || button.clicked() {
                        let unblocked_servers_remaining = servers
                            .iter()
                            .filter(|x| !desired_blocked_servers.has(x))
                            .count();

                        if !desired_blocked_servers.has(server) && unblocked_servers_remaining == 1
                        {
                            if button.clicked() {
                                log::warn!(
                                    "cannot block {} because all servers would be blocked",
                                    server.title.to_ascii_lowercase()
                                );
                            } else if button.secondary_clicked() {
                                desired_blocked_servers.solo(server);
                                selection_changed = true;
                            }
                        } else {
                            if button.clicked() {
                                desired_blocked_servers.toggle(server);
                                selection_changed = true;
                            } else if button.secondary_clicked() {
                                desired_blocked_servers.solo_invert(server);
                                selection_changed = true;
                            }
                        }

                        button.on_hover_text_at_pointer({
                            let text = if desired_blocked_servers.has(server)
                                != blocked_servers.has(server)
                            {
                                format!("{} (waiting for game to close..)", server.token)
                            } else {
                                if desired_blocked_servers.has(server) {
                                    format!("{} (blocked)", server.token)
                                } else {
                                    server.token.clone()
                                }
                            };

                            text
                        });
                    }
                }
            })
        });
    });

    selection_changed
}

/// draw a writable server
pub fn server_list_item(
    ui: &mut egui::Ui,
    server: &api::KnownServer,
    wants_blocked: bool,
    is_blocked: bool,
    i: usize,
    //
    pings: &crate::ping::Measurements,
    //
    theme: visuals::Theme,
) -> Option<egui::Response> {
    let mut b = None;

    ui.horizontal(|ui| {
        ui.scope(|ui| {
            let blocked = is_blocked;
            let pending = wants_blocked != is_blocked;

            if pending {
                if !wants_blocked {
                    // TODO
                }
            } else {
                if blocked {
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                        visuals::from_theme_alpha(theme, 0);
                    ui.style_mut().visuals.widgets.active.weak_bg_fill =
                        visuals::from_theme_alpha(theme, 40);
                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                        visuals::from_theme_alpha(theme, 20);

                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                        crate::visuals::color_secondary_faded(i);

                    ui.style_mut().visuals.override_text_color =
                        Some(ui.style_mut().visuals.weak_text_color());
                } else {
                    // ui.style_mut().visuals.override_text_color =
                    //     Some(crate::visuals::color_primary(i));

                    match theme {
                        visuals::Theme::Light => {
                            ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                                crate::visuals::color_inactive(i);
                            ui.style_mut().visuals.widgets.active.weak_bg_fill =
                                crate::visuals::color_active(i);
                            ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                                crate::visuals::color_hovered(i);
                        }
                        visuals::Theme::Dark => {
                            ui.style_mut().visuals.override_text_color =
                                Some(crate::visuals::color_inactive(i));
                        }
                    }

                    // #181818
                    // ui.style_mut().visuals.override_text_color =
                    //     Some(egui::Color32::from_rgb(24, 24, 24));
                }
            }

            ui.style_mut().interaction.selectable_labels = false;

            // Draw the title and ping ourselves so they fit even in mini mode.
            let button = ui.add(egui::Button::new("").min_size(egui::vec2(
                ui.available_width(),
                ui.spacing().interact_size.y,
            )));

            let rect = button.rect;

            b = Some(button);

            let mut child =
                ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink2(egui::vec2(8.0, 0.0))));

            child.horizontal(|ui| {
                // ui.label(if blocked { "<blocked />" } else { "" });

                if pending {
                    ui.add(egui::Spinner::new().size(16.));
                } else {
                    if blocked {
                        // ui.label("}: blocked :{");
                        ui.add(
                            egui::Image::new(assets::ICON_BAN)
                                .fit_to_exact_size(egui::vec2(16., 16.))
                                .tint(crate::visuals::color_primary(i)),
                        );
                    } else {
                        ui.add(
                            egui::Image::new(assets::ICON_STAR)
                                // TODO font size?
                                .fit_to_exact_size(egui::vec2(16., 16.))
                                .tint(crate::visuals::color_secondary_faded(i)),
                        );
                    }
                }

                {
                    let (icon, text, mut tooltip, tint) = {
                        match pings.get(&server.ping) {
                            // ping ok
                            Some(Ok(ms)) => (
                                Some(crate::ping_icon::ping_icon(*ms)),
                                format!("{ms:.0} ms"),
                                format!(
                                    "Average ICMP ping to {} (4 probes).\nRegional estimate; in-game latency may differ.",
                                    server.ping
                                ),
                                None,
                            ),

                            // ping error
                            Some(Err(e)) => (
                                None,
                                "n/a".to_string(),
                                format!("Ping to {} unavailable: {e}", server.ping),
                                Some(ui.visuals().weak_text_color()),
                            ),

                            // ping pending
                            None => (
                                Some(crate::ping_icon::ping_icon_cycle(ui.time())),
                                "... ms".to_string(),
                                format!("Measuring ICMP ping to {}...", server.ping),
                                Some(ui.visuals().weak_text_color()),
                            ),
                        }
                    };
                    if pings.is_measuring(&server.ping) && pings.get(&server.ping).is_some() {
                        tooltip.push_str("\nRefreshing; showing the previous measurement.");
                    }

                    let tag_color = tint.unwrap_or(ui.style().visuals.text_color());

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let label = ui.add(
                            egui::Label::new(egui::RichText::new(text).color(tag_color))
                                .wrap_mode(egui::TextWrapMode::Extend),
                        );
                        let tag = if pings.is_measuring(&server.ping) {
                            label.union(ui.add(egui::Spinner::new().size(16.0)))
                        } else if let Some(icon) = icon {
                            label.union(ui.add(
                                egui::Image::new(icon)
                                    .tint(tag_color)
                                    .fit_to_exact_size(egui::vec2(16.0, 16.0)),
                            ))
                        } else {
                            label
                        };
                        tag.on_hover_text_at_pointer(tooltip);

                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.add(egui::Label::new(&server.title).truncate())
                                .on_hover_text_at_pointer(&server.title);
                        });
                    });
                }
            });
        });
    });

    b
}
