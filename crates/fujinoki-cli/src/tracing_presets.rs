use once_cell::sync::Lazy;
use turbopack_binding::turbopack::trace_utils::tracing_presets;

pub static TRACING_OVERVIEW_TARGETS: Lazy<Vec<&str>> = Lazy::new(|| {
    vec![
        &tracing_presets::TRACING_OVERVIEW_TARGETS[..],
        &[
            "fujinoki_cli=info",
            "fujinoki_cli_utils=info",
            "fujinoki_dev_server=info",
            "fujinoki_ecmascript_plugins=info",
            "fujinoki_core=info",
            "fujinoki_node=info",
            "fujinoki_websocket=info",
        ],
        &["discord_api=info"],
    ]
    .concat()
});

pub static TRACING_FUJINOKI_TARGETS: Lazy<Vec<&str>> = Lazy::new(|| {
    [
        &TRACING_OVERVIEW_TARGETS[..],
        &[
            "fujinoki_cli=trace",
            "fujinoki_cli_utils=trace",
            "fujinoki_dev_server=trace",
            "fujinoki_ecmascript_plugins=trace",
            "fujinoki_core=trace",
            "fujinoki_node=trace",
            "fujinoki_websocket=trace",
        ],
    ]
    .concat()
});

pub static TRACING_DISCORD_TARGETS: Lazy<Vec<&str>> =
    Lazy::new(|| [&TRACING_FUJINOKI_TARGETS[..], &["discord_api=trace"]].concat());

pub static TRACING_TURBOPACK_TARGETS: Lazy<Vec<&str>> = Lazy::new(|| {
    [
        &TRACING_DISCORD_TARGETS[..],
        &tracing_presets::TRACING_TURBOPACK_TARGETS[..],
    ]
    .concat()
});

pub static TRACING_TURBO_TASKS_TARGETS: Lazy<Vec<&str>> = Lazy::new(|| {
    [
        &TRACING_TURBOPACK_TARGETS[..],
        &tracing_presets::TRACING_TURBO_TASKS_TARGETS[..],
    ]
    .concat()
});
