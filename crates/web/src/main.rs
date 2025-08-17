use dioxus::prelude::*;
use ui::Claim;
use ui::FAQ;
use ui::Logs;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const LOGO_IMAGE: Asset = asset!("/assets/logo.png");
const LOGO_LOGS_IMAGE: Asset = asset!("/assets/logo_logs.png");
const DEFAULT_THEME: Asset = asset!("/assets/default_theme.css");
const CUSTOM_THEME: Asset = asset!("/assets/custom_theme.css");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const OGIMAGE: Asset = asset!("/assets/OgImage.webp");
const GA: Asset = asset!("/assets/ga.js");

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
            alt_text: "A girl holding ETH crystal in a hand".to_string(),
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
            alt_text: "A girl looking at holographic display".to_string(),
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
                img { src: logo_img, alt: alt_text }
            }
            h1 { "{title}" }
        }
    }
}

#[component]
fn Head() -> Element {
    rsx! {
        document::Title { "Free Ethereum Sepolia Faucet - Claim Testnet ETH Fast" }
        document::Meta {
            name: "description",
            content: "Get free Sepolia ETH instantly with our open-source Ethereum faucet. Perfect for developers testing dApps and smart contracts on Sepolia testnet.",
        }
        document::Meta {
            name: "keywords",
            content: "ethereum faucet, sepolia faucet, free eth, claim sepolia, testnet eth, blockchain faucet, sepolia testnet, web3 faucet",
        }

        // openGraph
        document::Meta {
            property: "og:title",
            content: "Free Sepolia Ethereum Faucet - Claim Testnet ETH",
        }
        document::Meta {
            property: "og:description",
            content: "Claim free Sepolia ETH instantly. Open-source Ethereum faucet for Web3 developers.",
        }
        document::Meta { property: "og:type", content: "website" }
        document::Meta { property: "og:url", content: "https://faucet.free/" }
        document::Meta { property: "og:image", content: OGIMAGE }

        // Twitter
        document::Meta { name: "twitter:card", content: "summary_large_image" }
        document::Meta { name: "twitter:title", content: "Free Sepolia Ethereum Faucet" }
        document::Meta {
            name: "twitter:description",
            content: "Claim Sepolia ETH fast. Open-source, reliable faucet for developers.",
        }
        document::Meta { name: "twitter:image", content: OGIMAGE }


        document::Link { rel: "canonical", href: "https://faucet.free/" }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: DEFAULT_THEME }
        document::Link { rel: "stylesheet", href: CUSTOM_THEME }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        document::Script {
            r#async: true,
            src: "//gc.zgo.at/count.js",
            "data-goatcounter": "https://faucet.goatcounter.com/count",
        }

        document::Script {
            r#async: true,
            src: "https://www.googletagmanager.com/gtag/js?id=G-915ZM4GQ3S",
        }
        document::Script { src: GA }
    }
}

#[component]
fn App() -> Element {
    rsx! {
        Head {}
        Router::<Route> {}
    }
}
