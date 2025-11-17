#![allow(clippy::single_component_path_imports)]

use std::time::Duration;

use cgi;
use handlebars;
use http;
use serde;
use serde_json;
use ureq;
use url::form_urlencoded as url;

const X_CGI_CONTENT_TYPE: http::header::HeaderName =
    http::header::HeaderName::from_static("x-cgi-content-type");

fn get_param(url: &url::Parse, name: &str) -> Option<String> {
    url.clone()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.into_owned())
        .and_then(|s| if s.is_empty() { None } else { Some(s) })
}

struct CgiResult {
    status_code: http::StatusCode,
    body: String
}

type Cgi<T> = Result<T, CgiResult>;

const USER_AGENT: &str = "MyCGIClient/1.0";
const CT_FOR_DATA: &str = "application/json";
const CT_FOR_TEMPLATE: &str = "text/html,text/plain";

fn fetch(agent: &ureq::Agent, url: &str, accept: &str) -> Cgi<http::Response<ureq::Body>> {
    eprintln!("Fetching: {:?}", url);
    agent.get(url)
        .header(http::header::ACCEPT, accept)
        .header(http::header::USER_AGENT, USER_AGENT)
        .call()
        .to_cgi()
}

trait ToCgi<T> {
    fn to_cgi(self) -> Cgi<T>;
}

#[inline]
fn bad_media<T>() -> Cgi<T> {
    Err(CgiResult {
        status_code: http::StatusCode::UNSUPPORTED_MEDIA_TYPE,
        body: "Request body should be a UTF-8 encoded JSON/Form data".to_string(),
    })
}

impl <T> ToCgi<T> for Result<T, ureq::Error> {
    fn to_cgi(self) -> Cgi<T> {
        self.map_err(|err| {
            if let ureq::Error::StatusCode(code) = err {
                CgiResult {
                    status_code: http::StatusCode::from_u16(code).unwrap(),
                    body: err.to_string()
                }
            } else {
                eprintln!("ureq error: {:?}", err);
                CgiResult {
                    status_code: http::StatusCode::INTERNAL_SERVER_ERROR,
                    body: "Something went wrong".to_string(),
                }
            }
        })
    }
}

impl <T> ToCgi<T> for Result<T, handlebars::RenderError> {
    fn to_cgi(self) -> Cgi<T> {
        self.map_err(|err| {
            CgiResult {
                status_code: http::StatusCode::BAD_REQUEST,
                body: err.to_string(),
            }
        })
    }
}

impl <T> ToCgi<T> for Result<T, std::str::Utf8Error> {
    fn to_cgi(self) -> Cgi<T> {
        self.or(bad_media())
    }
}

impl <T> ToCgi<T> for Result<T, serde_json::Error> {
    fn to_cgi(self) -> Cgi<T> {
        self.map_err(|_| {
            CgiResult {
                status_code: http::StatusCode::BAD_REQUEST,
                body: "Incorrect JSON".to_string(),
            }
        })
    }
}

#[derive(serde::Deserialize)]
struct Params {
    #[serde(rename(deserialize = "t"))]
    template_url: String,
    #[serde(rename(deserialize = "d"))]
    data_url: String,
}

fn to_json_helper<'reg, 'rc>(
    h: &handlebars::Helper<'rc>,
    _: &'reg handlebars::Handlebars,
    _: &'rc handlebars::Context,
    _: &mut handlebars::RenderContext<'reg, 'rc>,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let datum: &serde_json::Value = match h.param(0) {
        None => return Ok(()),
        Some(p_and_v) => p_and_v.value(),
    };
    let raw = serde_json::to_string(&datum).unwrap();
    out.write(raw.as_str()).map_err(handlebars::RenderError::from)
}

impl Params {
    fn process(self)
               -> Cgi<String> {
        let agent = prepare_agent();
        let template = fetch(&agent, self.template_url.as_str(), CT_FOR_TEMPLATE)?
            .body_mut().read_to_string().to_cgi()?;
        let data = fetch(&agent, self.data_url.as_str(), CT_FOR_DATA)?
            .body_mut().read_json::<serde_json::Value>().to_cgi()?;
        let mut hb = handlebars::Handlebars::new();
        hb.register_helper("to_json", Box::new(to_json_helper));
        hb.render_template(template.as_str(), &data).to_cgi()
    }

    #[inline]
    fn from_urlencoded(query: &str) -> Cgi<Params> {
        let params = url::parse(query.as_bytes());
        let template_url = get_param(&params, "t");
        let data_url = get_param(&params, "d");
        if template_url.is_none() || data_url.is_none() {
            return Err(CgiResult {
                status_code: cgi::http::StatusCode::BAD_REQUEST,
                body: "Bad request".to_string(),
            });
        };
        Ok(Params {
            template_url: template_url.unwrap(),
            data_url: data_url.unwrap(),
        })
    }

    #[inline]
    fn from_json_body(body: &[u8]) -> Cgi<Params> {
        let decoded = std::str::from_utf8(body).to_cgi()?;
        let parsed: Params = serde_json::from_str(decoded).to_cgi()?;
        Ok(parsed)
    }

    #[inline]
    fn from_form_body(body: &[u8]) -> Cgi<Params> {
        let decoded = std::str::from_utf8(body).to_cgi()?;
        Params::from_urlencoded(decoded)
    }
}

#[inline]
fn prepare_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(true)
        .timeout_global(Some(Duration::from_secs(1)))
        .build()
        .into()
}

#[inline]
fn monadic<T> (body: T) -> impl FnOnce(cgi::Request) -> cgi::Response
where T: FnOnce(cgi::Request) -> Cgi<String>,
{
    |req: cgi::Request| -> cgi::Response {
        match body(req) {
            Ok(html) => cgi::html_response(http::StatusCode::OK, html),
            Err(res) => cgi::text_response(res.status_code, res.body)
        }
    }
}

#[inline]
fn get_content_type(headers: &http::HeaderMap) -> Cgi<&str> {
    let value =
        headers.get(http::header::CONTENT_TYPE)
        .or(headers.get(X_CGI_CONTENT_TYPE));
    if value.is_none() {
        Ok("")
    } else {
        let ct_value = value.unwrap().to_str().map_err(|_| {
            CgiResult { status_code: http::StatusCode::BAD_REQUEST, body: "Bad request".to_string() }
        })?;
        Ok(ct_value)
    }
}

const X_FORM: &str = "application/x-www-form-urlencoded";

fn main() {
    cgi::handle(monadic(|request: cgi::Request| -> Cgi<String> {
        let params: Params = match *request.method() {
            http::Method::GET => Params::from_urlencoded(request.uri().query().unwrap_or(""))?,
            http::Method::POST => {
                let content_type = get_content_type(request.headers())?;

                if content_type.is_empty() || content_type == "application/json" {
                    Params::from_json_body(request.body())?
                } else if content_type == X_FORM {
                    Params::from_form_body(request.body())?
                } else {
                    return bad_media()
                }
            },
            _ => return Err(CgiResult {
                status_code: http::StatusCode::METHOD_NOT_ALLOWED,
                body: "Method not allowed".to_string()
            })
        };
        params.process()
    }))
}
