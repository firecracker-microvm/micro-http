// Copyright (C) 2019 Alibaba Cloud. All rights reserved.
// Copyright © 2019 Intel Corporation
//
// SPDX-License-Identifier: Apache-2.0

use std::collections::hash_map::HashMap;

use crate::{MediaType, Request, Response, StatusCode, Version};

pub use crate::common::RouteError;

/// An HTTP endpoint handler interface
pub trait EndpointHandler<T>: Sync + Send {
    /// Handles an HTTP request.
    fn handle_request(&self, req: &Request, arg: &T) -> Response;
}

/// An HTTP routes structure.
pub struct HttpRoutes<T> {
    server_id: String,
    prefix: String,
    media_type: MediaType,
    /// routes is a hash table mapping endpoint URIs to their endpoint handlers.
    routes: HashMap<String, Box<dyn EndpointHandler<T> + Sync + Send>>,
}

impl<T: Send> HttpRoutes<T> {
    /// Create a http request router.
    pub fn new(server_id: String, prefix: String) -> Self {
        HttpRoutes {
            server_id,
            prefix,
            media_type: MediaType::ApplicationJson,
            routes: HashMap::new(),
        }
    }

    /// Register a request handler for a path.
    pub fn add_route(
        &mut self,
        path: String,
        handler: Box<dyn EndpointHandler<T> + Sync + Send>,
    ) -> Result<(), RouteError> {
        let full = format!("{}{}", self.prefix, path);
        if self.routes.contains_key(&full) {
            Err(RouteError::HandlerExist(full))
        } else {
            self.routes.insert(full, handler);
            Ok(())
        }
    }

    /// Handle an incoming http request and generate corresponding response.
    pub fn handle_http_request(&self, request: &Request, argument: T) -> Response {
        let path = request.uri().get_abs_path().to_string();
        let mut response = match self.routes.get(&path) {
            Some(route) => route.handle_request(&request, &argument),
            None => Response::new(Version::Http11, StatusCode::NotFound),
        };

        response.set_server(&self.server_id);
        response.set_content_type(self.media_type);
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct HandlerArg(bool);

    struct MockHandler {}

    impl EndpointHandler<HandlerArg> for MockHandler {
        fn handle_request(&self, _req: &Request, _arg: &HandlerArg) -> Response {
            Response::new(Version::Http11, StatusCode::OK)
        }
    }

    #[test]
    fn test_create_router() {
        let mut router = HttpRoutes::new("Mock_Server".to_string(), "/api/v1".to_string());
        let handler = MockHandler {};
        let res = router.add_route("/func1".to_string(), Box::new(handler));
        assert!(res.is_ok());
        let key = format!("{}{}", "/api/v1", "/func1");
        assert!(router.routes.contains_key(&key));

        let handler = MockHandler {};
        match router.add_route("/func1".to_string(), Box::new(handler)) {
            Err(RouteError::HandlerExist(_)) => {}
            _ => panic!("add_route() should return error for path with existing handler"),
        }

        let handler = MockHandler {};
        let res = router.add_route("/func2".to_string(), Box::new(handler));
        assert!(res.is_ok());
    }

    #[test]
    fn test_handle_http_request() {
        let mut router = HttpRoutes::new("Mock_Server".to_string(), "/api/v1".to_string());
        let handler = MockHandler {};
        router
            .add_route("/func1".to_string(), Box::new(handler))
            .unwrap();

        let request =
            Request::try_from(b"GET http://localhost/api/v1/func2 HTTP/1.1\r\n\r\n").unwrap();
        let arg = HandlerArg(true);
        let reply = router.handle_http_request(&request, arg);
        assert_eq!(reply.status(), StatusCode::NotFound);
    }
}
