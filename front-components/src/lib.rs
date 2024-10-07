use maud::{html, Markup};

pub fn rule_status(enabled: bool, id: u32) -> Markup {
    html! {
        @let color = { if enabled { "lime" } else { "rose" } };
        @let text = { if enabled { "Enabled" } else { "Disabled" } };

        button
            hx-post={ "http://127.0.0.1:9988/firewall/toggle/" (id) }
            hx-swap="outerHTML"
            .{ "bg-"(color)"-200" }
            .{ "border-"(color)"-800" }
            .{ "text-"(color)"-800" }
            .border.rounded-full
            .px-2
            { (text) }
    }
}
