use std::time::Duration;

use cgi;
use handlebars;
use http;
use serde_json;
use ureq;
use url::form_urlencoded as url;

fn get_param(url: &url::Parse, name: &str) -> Option<String> {
    url.clone()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.into_owned())
}

struct CgiResult {
    status_code: http::StatusCode,
    body: String
}

type Cgi<T> = Result<T, CgiResult>;

fn fetch(agent: &ureq::Agent, url: &str) -> Cgi<http::Response<ureq::Body>> {
    agent.get(url).call().to_cgi()
}

trait ToCgi<T> {
    fn to_cgi(self) -> Cgi<T>;
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

fn process(template_url: &str, data_url: &str) -> Cgi<String> {
    let agent = prepare_agent();
    let template = fetch(&agent, template_url)?
        .body_mut().read_to_string().to_cgi()?;
    let data = fetch(&agent, data_url)?
        .body_mut().read_json::<serde_json::Value>().to_cgi()?;
    let hb = handlebars::Handlebars::new();
    hb.render_template(template.as_str(), &data).to_cgi()
}

#[inline]
fn prepare_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(true)
        .timeout_global(Some(Duration::from_secs(1)))
        .build()
        .into()
}

fn main() {
    cgi::handle({ |request: cgi::Request| -> cgi::Response {
        if request.method() != cgi::http::Method::GET {
            return cgi::text_response(
                cgi::http::StatusCode::METHOD_NOT_ALLOWED,
                "Method not allowed"
            )
        };
        let query = request.uri().query().unwrap_or("");
        let params = url::parse(query.as_bytes());

        let template_url = get_param(&params, "t");
        let data_url = get_param(&params, "d");
        if template_url.is_none() || data_url.is_none() {
            return cgi::text_response(
                cgi::http::StatusCode::BAD_REQUEST,
                "Bad request"
            )
        };
        match process(
            template_url.unwrap().as_str(),
            data_url.unwrap().as_str()
        ) {
            Ok(body) => cgi::html_response(http::StatusCode::OK, body),
            Err(res) => cgi::text_response(res.status_code, res.body)
        }
    }})
}
