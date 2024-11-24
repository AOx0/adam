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

#[allow(non_snake_case)]
pub fn Padded(content: Markup) -> Markup {
    html! {
        div .space-5 .m-5 {
            (content)
        }
    }
}

#[allow(non_snake_case)]
pub fn Error(msg: &str) -> Markup {
    html! {
        div."w-full"."text-white"."bg-red-500" {
            div."container"."flex"."items-center"."justify-between"."px-6"."py-4"."mx-auto" {
                div."flex" {
                    svg."w-6"."h-6"."fill-current" viewBox="0 0 40 40" {
                        path d="M20 3.36667C10.8167 3.36667 3.3667 10.8167 3.3667 20C3.3667 29.1833 10.8167 36.6333 20 36.6333C29.1834 36.6333 36.6334 29.1833 36.6334 20C36.6334 10.8167 29.1834 3.36667 20 3.36667ZM19.1334 33.3333V22.9H13.3334L21.6667 6.66667V17.1H27.25L19.1334 33.3333Z" {}
                    }
                    p."mx-3" { (msg) }
                }
                button."p-1"."transition-colors"."duration-300"."transform"."rounded-md"."hover:bg-opacity-25"."hover:bg-gray-600"."focus:outline-none" {
                    svg."w-5"."h-5" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" {
                        path d="M6 18L18 6M6 6L18 18" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" {}
                    }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn Warning(msg: &str) -> Markup {
    html! {
        div."flex"."w-full"."max-w-sm"."overflow-hidden"."bg-white"."rounded-lg"."shadow-md"."dark:bg-gray-800" {
            div."flex"."items-center"."justify-center"."w-12"."bg-red-500" {
                svg."w-6"."h-6"."text-white"."fill-current" viewBox="0 0 40 40" xmlns="http://www.w3.org/2000/svg" {
                    path d="M20 3.36667C10.8167 3.36667 3.3667 10.8167 3.3667 20C3.3667 29.1833 10.8167 36.6333 20 36.6333C29.1834 36.6333 36.6334 29.1833 36.6334 20C36.6334 10.8167 29.1834 3.36667 20 3.36667ZM19.1334 33.3333V22.9H13.3334L21.6667 6.66667V17.1H27.25L19.1334 33.3333Z" {}
                }
            }
            div."px-4"."py-2".-"mx-3" {
                div."mx-3" {
                    span."font-semibold"."text-red-500"."dark:text-red-400" { "Error" }
                    p."text-sm"."text-gray-600"."dark:text-gray-200" {
                        (msg)
                    }
                }
            }
        }
    }
}

pub fn rule_status(enabled: bool, id: u32) -> Markup {
    html! {
        @let color = { if enabled { "[#69b3a2]" } else { "[#ff6347]" } };
        @let text = { if enabled { "Enabled" } else { "Disabled" } };

        // Trigger tailwindcss
        // bg-[#ff6347]/30 border-[#ff6347] text-[#ff6347]
        // bg-[#69b3a2]/30 border-[#69b3a2] text-[#69b3a2]
        // dark:bg-[#ff6347]/30 dark:border-[#ff6347] dark:text-[#ff6347]
        // dark:bg-[#69b3a2]/30 dark:border-[#69b3a2] dark:text-[#69b3a2]
        button
            hx-post={ "http://127.0.0.1:9988/firewall/rules/" (id) "/toggle" }
            hx-swap="outerHTML"
            .{ "bg-"(color)"/30" } .{ "dark:bg-"(color)"/30" }
            .{ "border-"(color) } .{ "dark:border-"(color) }
            .{ "text-"(color) } .{ "dark:text-"(color) }
            .border.rounded-full
            .px-2
            .text-sm
            { (text) }
    }
}

pub fn status(enabled: bool, toggle_url: &str) -> Markup {
    html! {
        @let color = { if enabled { "[#69b3a2]" } else { "[#ff6347]" } };
        @let text = { if enabled { "Enabled" } else { "Disabled" } };

        button
            hx-post=(toggle_url)
            hx-swap="outerHTML"
            .{ "bg-"(color)"/30" } .{ "dark:bg-"(color)"/30" }
            .{ "border-"(color) } .{ "dark:border-"(color) }
            .{ "text-"(color) } .{ "dark:text-"(color) }
            .border.rounded-full
            .px-2
            .text-sm
            { (text) }
    }
}
