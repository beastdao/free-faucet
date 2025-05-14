use dioxus::prelude::*;
use shared_types::LogEntry;
const LOGS_CSS: Asset = asset!("/assets/styling/logs.css");
use chrono::prelude::*;

fn prepare_log(entry: &LogEntry) -> Element {
    let (timestamp, status) = entry.key;
    let result = &entry.value.result;
    let prefix = "error running server function:";
    let fmt_result = result.replace(prefix, "");
    //modify
    let status_type = match status {
        1 => "success".to_string(),
        _ => {
            if fmt_result.contains("Cooldown is not ended")
                || fmt_result.contains("Unable to resolve the name: No name found")
            {
                "prevented".to_string()
            } else {
                "error".to_string()
            }
        }
    };

    let dt = Utc.timestamp_opt(timestamp as i64, 0).unwrap();
    rsx! {
        li {
            strong { class: "{status_type}", "{status_type}" }
            span { " {dt} - INPUT: [{entry.value.input.to_string()}], RESULT: {fmt_result} " }
        }
    }
}

#[component]
pub fn Logs() -> Element {
    let logs = use_resource(|| async move { server::shared::get_all_logs().await });
    let content = match &*logs.read_unchecked() {
        None => rsx! {
            li {
                strong { class: "prevented", "Loading..." }
            }
        },
        Some(Err(e)) => rsx! {
            li {
                strong { class: "error", "Error: {e}" }
            }
        },
        Some(Ok(entries)) if entries.is_empty() => {
            rsx! {
                li {
                    strong { class: "prevented", "Nothing to show" }
                }
            }
        }
        Some(Ok(entries)) => {
            rsx! {
                {entries.iter().map(prepare_log)}
            }
        }
    };

    rsx! {
        link { rel: "stylesheet", href: LOGS_CSS }
        div { id: "logs",
            ul { {content} }
        }
    }
}
