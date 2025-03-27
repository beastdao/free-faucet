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
    if input.is_empty() || !input.contains('@') {
        return false;
    }
    let parts: Vec<&str> = input.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let before_at = parts[0];
    let after_at = parts[1];
    before_at.len() >= 3 && after_at.len() >= 3
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
                                "Invalid input: Ensure it contains '@' and has at least 3 characters before and after."
                                    .to_string(),
                            );
                        response_state.set(ResponseState::Error);
                        return;
                    }
                    match server::claim_server(name.to_string()).await {
                        Ok(data) => {
                            response_state.set(ResponseState::Success);
                            response.set(format!("Sent! See tx on <a href='https://sepolia.etherscan.io/tx/{}' target='_blank'>Etherscan</a>", data));
                        }
                        Err(ServerFnError::ServerError(msg)) => {
                            response_state.set(ResponseState::Error);
                            response.set(format!("{}", msg));
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
                    dangerous_inner_html: "{response}"
                }
            }
        }
    }
}
