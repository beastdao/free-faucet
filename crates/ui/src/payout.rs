use dioxus::prelude::*;

const PAYOUT_CSS: Asset = asset!("/assets/styling/payout.css");

#[component]
pub fn Payout() -> Element {
    let payout_data = use_resource(|| async move { server::shared::get_payout_range_data().await });

    rsx! {
        document::Link { rel: "stylesheet", href: PAYOUT_CSS }
        div { id: "payout",
            match &*payout_data.read() {
                Some(Ok(data)) => {
                    // Parse values
                    let min = data.min.parse::<f64>().unwrap_or(0.0);
                    let max = data.max.parse::<f64>().unwrap_or(1.0);
                    let current = data.current.parse::<f64>().unwrap_or(min);

                    //dot position
                    let pos = if max > min {
                        ((current - min) / (max - min) * 100.0).clamp(0.0, 100.0)
                    } else {
                        0.0
                    };

                    rsx! {
                        h1 { "GET {current} SEPOLIA ETH NOW" }
                        div { id: "labels",
                            h3 { "{min} ETH" }
                            h3 { "{max} ETH" }
                        }

                        div { id: "track",
                            div {
                                id: "dot",
                                style: "left: {pos}%;",
                            }
                        }
                    }
                }
                Some(Err(err)) => rsx! { span { "Error: {err}" } },
                None => rsx! { span { "Loading..." } },
            }
        }
    }
}
