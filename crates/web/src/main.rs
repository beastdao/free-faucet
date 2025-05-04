use dioxus::prelude::*;
use ui::Claim;
use ui::FAQ;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const LOGO_IMAGE: Asset = asset!("/assets/logo.png");
const DEFAULT_THEME: Asset = asset!("/assets/default_theme.css");
const CUSTOM_THEME: Asset = asset!("/assets/custom_theme.css");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dioxus::LaunchBuilder::new()
        .with_context(server_only! {server::state::AppState::new()?})
        .launch(App);
    Ok(())
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

        div { id: "header",
            div { id: "logo",
                img {
                    src: LOGO_IMAGE,
                    alt: "A girl holding ETH crystal in a hand",
                }
            }
            h1 { "Ethereum Sepolia Faucet" }
        }
        Claim {}
        FAQ {}
    }
}
