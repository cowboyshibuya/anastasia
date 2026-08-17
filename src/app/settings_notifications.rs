//! Settings → Notifications: how a session asks for attention.

use gpui::KeyDownEvent;

use super::*;

/// One switch on the page: what it reads, and what flipping it does.
struct NotificationToggle {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    enabled: bool,
    apply: fn(&mut Waku, bool),
}

impl Waku {
    fn set_notification_settings(
        &mut self,
        apply: impl FnOnce(&mut Waku, bool),
        enabled: bool,
        cx: &mut Context<Self>,
    ) {
        apply(self, enabled);
        self.save();
        cx.notify();
    }

    pub(super) fn render_notifications_settings(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = Theme::current(cx);
        let settings = self.state.notifications;

        let toggles = [
            NotificationToggle {
                id: "notification-sound",
                title: "notifications.sound",
                description: "notifications.sound_description",
                enabled: settings.sound_enabled,
                apply: |this, enabled| this.state.notifications.sound_enabled = enabled,
            },
            NotificationToggle {
                id: "notification-banners",
                title: "notifications.banners",
                description: "notifications.banners_description",
                enabled: settings.banners_enabled,
                apply: |this, enabled| this.state.notifications.banners_enabled = enabled,
            },
            NotificationToggle {
                id: "notification-background-only",
                title: "notifications.background_only",
                description: "notifications.background_only_description",
                enabled: settings.background_only,
                apply: |this, enabled| this.state.notifications.background_only = enabled,
            },
        ];

        let rows = toggles.into_iter().enumerate().map(|(index, toggle)| {
            let NotificationToggle {
                id,
                title,
                description,
                enabled,
                apply,
            } = toggle;

            // The same switch the General page uses, so the two read as one
            // settings surface.
            let control = div()
                .id(id)
                .tab_index(0)
                .focus_visible(|style| style.border_color(theme.accent))
                .w(px(36.0))
                .h(px(20.0))
                .p(px(2.0))
                .flex_none()
                .rounded_full()
                .cursor_default()
                .bg(if enabled { theme.inverse } else { theme.inset })
                .border_1()
                .border_color(if enabled {
                    theme.inverse
                } else {
                    theme.border_strong
                })
                .flex()
                .items_center()
                .when(enabled, |element| element.justify_end())
                .child(div().w(px(14.0)).h(px(14.0)).rounded_full().bg(if enabled {
                    theme.on_inverse
                } else {
                    theme.text_tertiary
                }))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_notification_settings(apply, !enabled, cx);
                }))
                .on_key_down(cx.listener(move |this, event: &KeyDownEvent, _, cx| {
                    if !event.keystroke.modifiers.modified()
                        && matches!(event.keystroke.key.as_str(), "enter" | "space")
                    {
                        this.set_notification_settings(apply, !enabled, cx);
                        cx.stop_propagation();
                    }
                }));

            let row = div()
                .w_full()
                .min_h(px(60.0))
                .px(px(20.0))
                .py(px(12.0))
                .flex()
                .items_center()
                .gap(px(24.0))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(
                            div()
                                .text_size(px(13.5))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text)
                                .child(crate::i18n::translate(title)),
                        )
                        .child(
                            div()
                                .mt(px(5.0))
                                .text_size(px(12.5))
                                .line_height(px(18.0))
                                .text_color(theme.text_secondary)
                                .child(crate::i18n::translate(description)),
                        ),
                )
                .child(control);

            if index == 0 {
                row.into_any_element()
            } else {
                div()
                    .child(div().mx(px(20.0)).h(px(1.0)).bg(theme.border))
                    .child(row)
                    .into_any_element()
            }
        });

        div()
            .child(
                div()
                    .mt(px(15.0))
                    .w_full()
                    .flex()
                    .flex_col()
                    .rounded(px(13.0))
                    .overflow_hidden()
                    .bg(theme.raised)
                    .children(rows),
            )
            .child(
                div()
                    .mt(px(10.0))
                    .px(px(4.0))
                    .text_size(px(12.0))
                    .line_height(px(17.0))
                    .text_color(theme.text_tertiary)
                    .child(tr!("notifications.footnote")),
            )
            .into_any_element()
    }
}
