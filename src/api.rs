#[cfg(feature = "blocking")]
use reqwest::blocking::Request as BlockingRequest;
use reqwest::{Method, Request, Url};
use std::convert::From;

const URL_ENDPOINT: &str = "https://www.alphavantage.co/query";

pub(crate) struct APIRequestBuilder {
    key: String,
}

impl APIRequestBuilder {
    pub(crate) fn new(key: &str) -> APIRequestBuilder {
        APIRequestBuilder {
            key: String::from(key),
        }
    }

    pub(crate) fn create<'a>(
        &'a self,
        function: &'a str,
        params: &'a [(&'a str, &'a str)],
    ) -> APIRequest<'a> {
        APIRequest::new(&self.key, function, params)
    }
}

pub(crate) struct APIRequest<'a> {
    key: &'a str,
    function: &'a str,
    params: &'a [(&'a str, &'a str)],
}

impl<'a> APIRequest<'a> {
    fn new(key: &'a str, function: &'a str, params: &'a [(&'a str, &'a str)]) -> APIRequest<'a> {
        APIRequest {
            key,
            function,
            params,
        }
    }

    fn url(&self) -> Url {
        let mut url = Url::parse(URL_ENDPOINT).unwrap();
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("function", self.function);
            query.append_pair("apikey", self.key);
            for param in self.params {
                query.append_pair(param.0, param.1);
            }
        }
        url
    }
}

impl From<APIRequest<'_>> for Request {
    fn from(request: APIRequest) -> Self {
        reqwest::Request::new(Method::GET, request.url())
    }
}

#[cfg(feature = "blocking")]
impl<'a> From<APIRequest<'a>> for BlockingRequest {
    fn from(request: APIRequest) -> Self {
        reqwest::blocking::Request::new(Method::GET, request.url())
    }
}
