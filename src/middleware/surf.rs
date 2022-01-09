use crate::{CacheError, HttpResponse, Middleware, Result};

use std::{collections::HashMap, convert::TryInto, str::FromStr, time::SystemTime};

use http::{header::CACHE_CONTROL, request::Parts};
use http_cache_semantics::{AfterResponse, BeforeRequest, CachePolicy};
use url::Url;

pub(crate) struct SurfMiddleware<'a> {
    pub req: surf::Request,
    pub client: surf::Client,
    pub next: surf::middleware::Next<'a>,
}

#[async_trait::async_trait]
impl Middleware for SurfMiddleware<'_> {
    fn is_method_get_head(&self) -> bool {
        self.req.method() == http_types::Method::Get
            || self.req.method() == http_types::Method::Head
    }
    fn new_policy(&self, response: &HttpResponse) -> Result<CachePolicy> {
        Ok(CachePolicy::new(
            &self.get_request_parts()?,
            &response.get_parts()?,
        ))
    }
    fn update_request_headers(&mut self, parts: Parts) -> Result<()> {
        for header in parts.headers.iter() {
            let value = match http_types::headers::HeaderValue::from_str(header.1.to_str()?) {
                Ok(v) => v,
                Err(_e) => return Err(CacheError::BadHeader),
            };
            self.req.set_header(header.0.as_str(), value);
        }
        Ok(())
    }
    fn set_no_cache(&mut self) -> Result<()> {
        self.req.insert_header(CACHE_CONTROL.as_str(), "no-cache");
        Ok(())
    }
    fn get_request_parts(&self) -> Result<Parts> {
        let mut headers = http::HeaderMap::new();
        for header in self.req.iter() {
            headers.insert(
                http::header::HeaderName::from_str(header.0.as_str())?,
                http::HeaderValue::from_str(header.1.as_str())?,
            );
        }
        let uri = http::Uri::from_str(self.req.url().as_str())?;
        let method = http::Method::from_str(self.req.method().as_ref())?;
        let mut converted = http::request::Request::new(());
        converted.headers_mut().clone_from(&headers);
        converted.uri_mut().clone_from(&uri);
        converted.method_mut().clone_from(&method);
        let parts = converted.into_parts();
        Ok(parts.0)
    }
    fn before_request(&self, policy: &CachePolicy) -> Result<BeforeRequest> {
        Ok(policy.before_request(&self.get_request_parts()?, SystemTime::now()))
    }
    fn after_response(
        &self,
        policy: &CachePolicy,
        response: &HttpResponse,
    ) -> Result<AfterResponse> {
        Ok(policy.after_response(
            &self.get_request_parts()?,
            &response.get_parts()?,
            SystemTime::now(),
        ))
    }
    fn url(&self) -> Result<&Url> {
        Ok(self.req.url())
    }
    fn method(&self) -> Result<String> {
        Ok(self.req.method().as_ref().to_string())
    }
    async fn remote_fetch(&self) -> Result<HttpResponse> {
        let url = self.req.url().clone();
        let mut res = self
            .next
            .run(self.req.clone(), self.client.clone())
            .await
            .unwrap();
        let mut headers = HashMap::new();
        for header in res.iter() {
            headers.insert(header.0.as_str().to_owned(), header.1.as_str().to_owned());
        }
        let status = res.status().into();
        let version = res.version().unwrap_or(http_types::Version::Http1_1);
        let body: Vec<u8> = res.body_bytes().await.unwrap();
        Ok(HttpResponse {
            body,
            headers,
            status,
            url,
            version: version.try_into()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{CACacheManager, Cache, CacheManager, CacheMode};
    use mockito::mock;
    use surf::{http::Method, Client, Request, Url};

    #[cfg(feature = "client-surf")]
    #[async_std::test]
    async fn default_mode() -> surf::Result<()> {
        let m = mock("GET", "/")
            .with_status(200)
            .with_header("cache-control", "max-age=86400, public")
            .with_body("test")
            .create();
        let url = format!("{}/", &mockito::server_url());
        let manager = CACacheManager::default();
        let path = manager.path.clone();
        let key = format!("GET:{}", &url);
        let req = Request::new(Method::Get, Url::parse(&url)?);

        // Make sure the record doesn't already exist
        manager.delete("GET", &Url::parse(&url)?).await?;

        // Construct Surf client with cache defaults
        let client = Client::new().with(Cache {
            mode: CacheMode::Default,
            cache_manager: CACacheManager::default(),
        });

        // Cold pass to load cache
        client.send(req.clone()).await?;
        m.assert();

        // Try to load cached object
        let data = cacache::read(&path, &key).await;
        assert!(data.is_ok());
        Ok(())
    }
}