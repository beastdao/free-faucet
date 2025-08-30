use dioxus::prelude::*;

const CLAIM_CSS: Asset = asset!("/assets/styling/claim.css");

#[derive(PartialEq)]
enum ResponseState {
    Loading,
    Success,
    Error,
    None,
}

fn is_valid_input(input: &str) -> bool {
    input
        .split_once('@')
        .map_or(false, |(before_at, after_at)| {
            (3..=15).contains(&before_at.len()) && (3..=10).contains(&after_at.len())
        })
}

#[component]
pub fn Claim() -> Element {
    let mut response = use_signal(|| String::new());
    let mut name = use_signal(|| String::new());
    let mut response_state = use_signal(|| ResponseState::None);

    use_effect(move || {
        name.set(String::new());
    });

    rsx! {
        document::Link { rel: "stylesheet", href: CLAIM_CSS }

        div { id: "claim",

            form {
                onsubmit: move |_| async move {
                    response_state.set(ResponseState::Loading);
                    if !is_valid_input(&name.to_string()) {
                        response
                            .set(
                                "Invalid input: Use name@tln (name 3–15 chars, tln 3-10)."
                                    .to_string(),
                            );
                        response_state.set(ResponseState::Error);
                        return;
                    }
                    match server::shared::claim_server(name.to_string()).await {
                        Ok(data) => {
                            response_state.set(ResponseState::Success);
                            response
                                .set(
                                    format!(
                                        "Sent! See tx on <a href='https://sepolia.etherscan.io/tx/{}' target='_blank'>Etherscan</a>",
                                        data,
                                    ),
                                );
                        }
                        Err(ServerFnError::ServerError(msg)) => {
                            response_state.set(ResponseState::Error);
                            if msg.contains("Unable to resolve the name: No name found") {
                                let clean_name = name
                                    .to_string()
                                    .split('@')
                                    .next()
                                    .unwrap_or_default()
                                    .to_string();
                                let suggested_name = format!("{}@eth", clean_name.to_lowercase());
                                response
                                    .set(
                                        format!(
                                            "{}. <br> <a href='https://app.0xname.foo/RegisterNameFinal/{}' target='_blank'> > GET FREE {} < </a>",
                                            msg,
                                            suggested_name,
                                            suggested_name,
                                        ),
                                    );
                            } else {
                                response.set(msg);
                            }
                        }
                        Err(_) => {
                            response_state.set(ResponseState::Error);
                            response.set("An unexpected error occurred.".to_string());
                        }
                    };
                },
                input {
                    placeholder: "Type your 0xNAME like alice@eth",
                    oninput: move |event| name.set(event.value()),
                    value: name,
                }
                button {
                    r#type: "submit",
                    disabled: *response_state.read() == ResponseState::Loading,
                    "Claim SepETH"
                }
            }

            if *response_state.read() == ResponseState::Loading {
                p { class: "loading", "Processing your request..." }
            } else if !response().is_empty() {
                p {
                    class: match *response_state.read() {
                        ResponseState::Success => "success",
                        ResponseState::Error => "error",
                        _ => "",
                    },
                    dangerous_inner_html: "{response}",
                }
            }
        }
    }
}
