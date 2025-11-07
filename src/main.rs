use cgi;
use handlebars;
use serde_json;
use reqwest::blocking as req;
use http;
use url;

// TODO: try 'ureq'

fn get_param(url: &url::Url, name: &str) -> Option<String> {
    url.query_pairs()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.into_owned())
}

fn fetch(url: &str) -> Result<req::Response, cgi::Response> {
    req::get(url).map_err(|err| {
        cgi::text_response(
            err.status().unwrap(),
            err.without_url().to_string(),
        )
    })
}

trait ToCgi<T> {
    fn to_cgi(self) -> Result<T, cgi::Response>;
}

impl <T> ToCgi<T> for Result<T, reqwest::Error> {
    fn to_cgi(self) -> Result<T, cgi::Response> {
        self.map_err(|err| {
            cgi::text_response(
            http::StatusCode::BAD_REQUEST,
            err.without_url().to_string()
            )
        })
    }
}

impl <T> ToCgi<T> for Result<T, handlebars::RenderError> {
    fn to_cgi(self) -> Result<T, cgi::Response> {
        self.map_err(|err| {
            cgi::text_response(
            http::StatusCode::BAD_REQUEST,
            err.to_string()
            )
        })
    }
}

fn process(template_url: &str, data_url: &str) -> Result<String, cgi::Response> {
    let template = fetch(template_url)?.text().to_cgi()?;
    let data: serde_json::Value = fetch(data_url)?.json().to_cgi()?;
    let hb = handlebars::Handlebars::new();
    hb.render_template(template.as_str(), &data).to_cgi()
}

fn main() {
    cgi::handle({ |request: cgi::Request| -> cgi::Response {
        if request.method() != cgi::http::Method::GET {
            return cgi::text_response(
                cgi::http::StatusCode::METHOD_NOT_ALLOWED,
                "Method not allowed"
            )
        };
        // TODO: ugly hack, need to make it properly!
        let query = format!("http://mock{}", request.uri());
        let url = url::Url::parse( query.as_str()).unwrap();

        let template_url = get_param(&url, "t");
        let data_url = get_param(&url, "d");
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
            Ok(result) => cgi::html_response(http::StatusCode::OK, result),
            Err(res) => res,
        }
    }})
}
