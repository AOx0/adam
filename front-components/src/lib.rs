use maud::{html, Markup};

#[allow(non_snake_case)]
pub fn Ref(title: impl maud::Render, href: &str) -> Markup {
    html! {
        button
            .text-sm.font-medium."space-x-5"
            .text-foreground.transition-colors
        {
            p."hover:text-foreground/80"."text-foreground/60"
            hx-boost="true"
            hx-push-url="true"
            hx-target="#main"
            hx-get={ (href) } { (title) }
        }
    }
}

pub fn rule_status(enabled: bool, id: u32) -> Markup {
    html! {
        @let color = { if enabled { "lime" } else { "rose" } };
        @let text = { if enabled { "Enabled" } else { "Disabled" } };

        button
            hx-post={ "http://127.0.0.1:9988/firewall/rules/" (id) "/toggle" }
            hx-swap="outerHTML"
            .{ "bg-"(color)"-200" }
            .{ "border-"(color)"-800" }
            .{ "text-"(color)"-800" }
            .border.rounded-full
            .px-2
            .text-sm
            { (text) }
    }
}
