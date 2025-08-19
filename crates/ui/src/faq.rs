use dioxus::prelude::*;

const FAQ_CSS: Asset = asset!("/assets/styling/faq.css");

#[component]
pub fn FAQ() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: FAQ_CSS }
        div { id: "faq",
            dl {
                dt { "How does the faucet work?" }
                dd {
                    "To request funds, simply enter your 0xNAME with any TLN (Top Level Name) such as "
                    a { href: "https://app.0xname.foo/eth", target: "_blank", "myname@eth" }
                    " or "
                    a {
                        href: "https://app.0xname.foo/sepolia",
                        target: "_blank",
                        "myname@sepolia"
                    }
                    " and hit 'Claim SepETH'. "
                }
                dt { "How much can I claim?" }
                dd { "You can request 0.05 Sepolia ETH every 24 hours. " }
                dt { "What is 0xNAME and where do I get it? " }
                dd {
                    "0xNAME is a FREE public good personal names on the Ethereum blockchain. You can get your free name, such as alice@eth or bob@yourdao, at "
                    a { href: "https://app.0xname.foo", target: "_blank", "app.0xname.foo" }
                }
                dt { "Is the faucet open source?" }
                dd {
                    "Yes, the faucet is open source. You can find the code on "
                    a {
                        href: "https://github.com/beastdao/free-faucet",
                        target: "_blank",
                        "GitHub"
                    }
                    ". If you find a bug, please submit an issue. If you like the project, you can give it a star or fork and run your own faucet."
                }
                dt { "Does the faucet have logs?" }

                dd {
                    "Yes, the faucet logs are available here: "
                    a { href: "/logs", "Logs" }
                }

                dt { "Got more questions or feedback?" }
                dd {
                    "Help us improve! Join the conversation on our "
                    a {
                        href: "https://discord.com/invite/McqF7vyCWx",
                        target: "_blank",
                        "Discord server"
                    }
                }
                dt { "How can I support the faucet?" }
                dd {
                    "You can support the faucet by donating Sepolia ETH to the address: "
                    code { "0xf0E5D3Cc05206987a125afC404b719e54Fa942a8" }
                }
            }
        }
    }
}
