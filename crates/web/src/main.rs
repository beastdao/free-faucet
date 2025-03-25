use dioxus::prelude::*;
use ui::Claim;
use ui::FAQ;

const FAVICON: Asset = asset!("/assets/favicon.ico");
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
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        div { id: "header",
            div { id: "orb" }
            h1 { "0xNAME faucet" }
        }
        Claim {}
        FAQ {}
    }
}
