use std::collections::HashMap;

use anyhow::{bail, Result};
use reqwest::Method;
use serde_json::Value as JsonValue;
use turbopack_binding::{
    turbo::{
        tasks as turbo_tasks,
        tasks::{duration_span, RcStr, Vc},
        tasks_fetch::{
            FetchError, FetchErrorKind, FetchResult, HttpResponse, HttpResponseBody,
            OptionProxyConfig, ProxyConfig,
        },
    },
    turbopack::core::issue::StyledString,
};

use super::routes::Route;

#[turbo_tasks::value(transparent)]
pub struct OptionHashMap(Option<HashMap<RcStr, RcStr>>);

#[turbo_tasks::value(transparent)]
pub struct OptionQueries(Option<Vec<(RcStr, RcStr)>>);

#[turbo_tasks::value]
pub struct Http {
    base_url: RcStr,
}

#[turbo_tasks::value_impl]
impl Http {
    #[turbo_tasks::function]
    pub fn new(base_url: RcStr) -> Vc<Self> {
        Http { base_url }.cell()
    }

    #[turbo_tasks::function]
    async fn make_url(self: Vc<Self>, route: Vc<Route>) -> Result<Vc<RcStr>> {
        let url = &*self.await?.base_url;
        let slash = (url.ends_with('/'), route.await?.endpoint.starts_with('/'));

        Ok(Vc::cell(
            match slash {
                (true, true) => {
                    format!("{url}{}", &route.await?.endpoint[1..])
                }
                (true, false) | (false, true) => {
                    format!("{url}{}", route.await?.endpoint)
                }
                (false, false) => {
                    format!("{url}/{}", route.await?.endpoint)
                }
            }
            .into(),
        ))
    }

    #[turbo_tasks::function]
    async fn validate_request(
        self: Vc<Self>,
        method: RcStr,
        route: Vc<Route>,
    ) -> Result<Vc<RcStr>> {
        if !route.await?.methods.contains(&method) {
            bail!("{} cannot make a {method} request", route.await?.endpoint);
        }

        Ok(self.make_url(route))
    }

    #[turbo_tasks::function(network)]
    pub async fn get(
        self: Vc<Self>,
        route: Vc<Route>,
        headers: Vc<OptionHashMap>,
        queries: Vc<OptionQueries>,
    ) -> Result<Vc<FetchResult>> {
        let url = self.validate_request(Method::GET.to_string().into(), route);

        Ok(without_body(
            Method::GET.to_string().into(),
            url,
            headers,
            queries,
            Vc::cell(None),
            Vc::cell(None),
        ))
    }

    #[turbo_tasks::function(network)]
    pub async fn post(
        self: Vc<Self>,
        route: Vc<Route>,
        json: Vc<JsonValue>,
        headers: Vc<OptionHashMap>,
    ) -> Result<Vc<FetchResult>> {
        let url = self.validate_request(Method::POST.to_string().into(), route);

        Ok(with_body(
            Method::POST.to_string().into(),
            url,
            json,
            headers,
            Vc::cell(None),
            Vc::cell(None),
        ))
    }

    #[turbo_tasks::function(network)]
    pub async fn put(
        self: Vc<Self>,
        route: Vc<Route>,
        json: Vc<JsonValue>,
        headers: Vc<OptionHashMap>,
    ) -> Result<Vc<FetchResult>> {
        let url = self.validate_request(Method::PATCH.to_string().into(), route);

        Ok(with_body(
            Method::PUT.to_string().into(),
            url,
            json,
            headers,
            Vc::cell(None),
            Vc::cell(None),
        ))
    }

    #[turbo_tasks::function(network)]
    pub async fn patch(
        self: Vc<Self>,
        route: Vc<Route>,
        json: Vc<JsonValue>,
        headers: Vc<OptionHashMap>,
    ) -> Result<Vc<FetchResult>> {
        let url = self.validate_request(Method::PATCH.to_string().into(), route);

        Ok(with_body(
            Method::PATCH.to_string().into(),
            url,
            json,
            headers,
            Vc::cell(None),
            Vc::cell(None),
        ))
    }

    #[turbo_tasks::function(network)]
    pub async fn delete(
        self: Vc<Self>,
        route: Vc<Route>,
        headers: Vc<OptionHashMap>,
        queries: Vc<OptionQueries>,
    ) -> Result<Vc<FetchResult>> {
        let url = self.validate_request(Method::DELETE.to_string().into(), route);

        Ok(without_body(
            Method::DELETE.to_string().into(),
            url,
            headers,
            queries,
            Vc::cell(None),
            Vc::cell(None),
        ))
    }
}

// This is basically just the [turbo_tasks_fetch::fetch] function but with some
// extra arguments
#[turbo_tasks::function(network)]
pub async fn without_body(
    method: RcStr,
    url: Vc<RcStr>,
    headers: Vc<OptionHashMap>,
    queries: Vc<OptionQueries>,
    user_agent: Vc<Option<RcStr>>,
    proxy_option: Vc<OptionProxyConfig>,
) -> Result<Vc<FetchResult>> {
    let url = &*url.await?;
    let headers = &*headers.await?;
    let queries = &*queries.await?;
    let user_agent = &*user_agent.await?;
    let proxy_option = &*proxy_option.await?;
    let guard = Box::new(duration_span!(
        "HTTP Request without body",
        method = display(method.to_string()),
        url = display(url.to_string())
    ));

    let client_builder = reqwest::Client::builder();
    let client_builder = match proxy_option {
        Some(ProxyConfig::Http(proxy)) => client_builder.proxy(reqwest::Proxy::http(proxy)?),
        Some(ProxyConfig::Https(proxy)) => client_builder.proxy(reqwest::Proxy::https(proxy)?),
        _ => client_builder,
    };

    let client = client_builder.build()?;

    let mut builder = match method.as_str() {
        "GET" => client.get(url.as_str()),
        "DELETE" => client.delete(url.as_str()),
        _ => bail!("Invalid method"),
    };
    if let Some(user_agent) = user_agent {
        builder = builder.header("User-Agent", user_agent.as_str());
    }

    if let Some(headers) = headers {
        for (key, value) in headers {
            builder = builder.header(key.as_str(), value.as_str());
        }
    };

    if let Some(queries) = queries {
        for query in queries.iter() {
            builder = builder.query(query);
        }
    }

    let response = builder.send().await.and_then(|r| r.error_for_status());

    drop(guard);

    match response {
        Ok(response) => {
            let status = response.status().as_u16();
            let body = response.bytes().await?.to_vec();

            Ok(Vc::cell(Ok(HttpResponse {
                status,
                body: HttpResponseBody::cell(HttpResponseBody(body)),
            }
            .cell())))
        }
        Err(err) => Ok(Vc::cell(Err(from_reqwest_error(&err, url).cell()))),
    }
}

#[turbo_tasks::function(network)]
pub async fn with_body(
    method: RcStr,
    url: Vc<RcStr>,
    json: Vc<JsonValue>,
    headers: Vc<OptionHashMap>,
    user_agent: Vc<Option<RcStr>>,
    proxy_option: Vc<OptionProxyConfig>,
) -> Result<Vc<FetchResult>> {
    let url = &*url.await?;
    let headers = &*headers.await?;
    let user_agent = &*user_agent.await?;
    let proxy_option = &*proxy_option.await?;
    let guard = Box::new(duration_span!(
        "HTTP Request with body",
        method = display(method.to_string()),
        url = display(url.to_string())
    ));

    let client_builder = reqwest::Client::builder();
    let client_builder = match proxy_option {
        Some(ProxyConfig::Http(proxy)) => client_builder.proxy(reqwest::Proxy::http(proxy)?),
        Some(ProxyConfig::Https(proxy)) => client_builder.proxy(reqwest::Proxy::https(proxy)?),
        _ => client_builder,
    };

    let client = client_builder.build()?;

    let mut builder = match method.as_str() {
        "POST" => client.post(url.as_str()),
        "PATCH" => client.patch(url.as_str()),
        "PUT" => client.put(url.as_str()),
        _ => bail!("Invalid method"),
    };
    if let Some(user_agent) = user_agent {
        builder = builder.header("User-Agent", user_agent.as_str());
    }

    if let Some(headers) = headers {
        for (key, value) in headers {
            builder = builder.header(key.as_str(), value.as_str());
        }
    };

    let response = builder
        .json(&*json.await?)
        .send()
        .await
        .and_then(|r| r.error_for_status());

    drop(guard);

    match response {
        Ok(response) => {
            let status = response.status().as_u16();
            let body = response.bytes().await?.to_vec();

            Ok(Vc::cell(Ok(HttpResponse {
                status,
                body: HttpResponseBody::cell(HttpResponseBody(body)),
            }
            .cell())))
        }
        Err(err) => Ok(Vc::cell(Err(
            from_reqwest_error(&err.without_url(), url).cell()
        ))),
    }
}

fn from_reqwest_error(error: &reqwest::Error, url: &str) -> FetchError {
    let kind = if error.is_connect() {
        FetchErrorKind::Connect
    } else if error.is_timeout() {
        FetchErrorKind::Timeout
    } else if let Some(status) = error.status() {
        FetchErrorKind::Status(status.as_u16())
    } else {
        FetchErrorKind::Other
    };

    FetchError {
        // `without_url` because the url is included in another property (url)
        detail: StyledString::Text(error.to_string().into()).cell(),
        url: Vc::cell(url.into()),
        kind: kind.into(),
    }
}

#[turbo_tasks::function]
pub async fn fetch_error_to_string(fetch_error: Vc<FetchError>) -> Result<Vc<RcStr>> {
    let fetch_error = &*fetch_error.await?;
    let detail = &*fetch_error.detail.await?;
    let string = match detail {
        StyledString::Text(text) => text,
        _ => unreachable!(),
    };

    Ok(Vc::cell(string.clone()))
}
