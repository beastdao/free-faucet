use dioxus::prelude::*;
use ui::Claim;
use ui::Logs;
use ui::FAQ;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const LOGO_IMAGE: Asset = asset!("/assets/logo.png");
const LOGO_LOGS_IMAGE: Asset = asset!("/assets/logo_logs.png");
const DEFAULT_THEME: Asset = asset!("/assets/default_theme.css");
const CUSTOM_THEME: Asset = asset!("/assets/custom_theme.css");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/logs")]
    LogsPage {},
}

#[component]
fn Home() -> Element {
    rsx! {
        Header {
            logo_img: LOGO_IMAGE,
            title: "Ethereum Sepolia Faucet".to_string(),
            alt_text: "A girl holding ETH crystal in a hand".to_string()
        }
        Claim {}
        FAQ {}
    }
}

#[component]
fn LogsPage() -> Element {
    rsx! {
        Header {
            logo_img: LOGO_LOGS_IMAGE,
            title: "Ethereum Sepolia Faucet".to_string(),
            alt_text: "A girl looking at holographic display".to_string()
        }
        Logs {}
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dioxus::LaunchBuilder::new()
        .with_context(server_only! {server::state::AppState::new()?})
        .launch(App);
    Ok(())
}

#[component]
fn Header(logo_img: Asset, title: String, alt_text: String) -> Element {
    rsx! {
        div { id: "header",
            div { id: "logo",
                img {
                    src: logo_img,
                    alt: alt_text,
                }
            }
            h1 { "{title}" }
        }
    }
}

#[component]
fn App() -> Element {
    rsx! {
        head {
            title { "0xNAME Sepolia ETH Faucet" }
            meta {
                name: "description",
                content: "Sepolia Testnet ETH Faucet powered by 0xname public good names on Ethereum blockchain",
            }
        }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: DEFAULT_THEME }
        document::Link { rel: "stylesheet", href: CUSTOM_THEME }
        document::Link { rel: "stylesheet", href: MAIN_CSS }


        Router::<Route> {}
    }
}
