// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{Display, Error, Formatter};

pub mod headers;

pub mod ascii {
    pub const CR: u8 = b'\r';
    pub const COLON: u8 = b':';
    pub const LF: u8 = b'\n';
    pub const SP: u8 = b' ';
    pub const CRLF_LEN: usize = 2;
}

/// Errors associated with parsing the HTTP Request from a u8 slice.
#[derive(Debug, PartialEq)]
pub enum RequestError {
    /// The HTTP Method is not supported or it is invalid.
    InvalidHttpMethod(&'static str),
    /// Request URI is invalid.
    InvalidUri(&'static str),
    /// The HTTP Version in the Request is not supported or it is invalid.
    InvalidHttpVersion(&'static str),
    /// The header specified may be valid, but is not supported by this HTTP implementation.
    UnsupportedHeader,
    /// Header specified is invalid.
    InvalidHeader,
    /// The Request is invalid and cannot be served.
    InvalidRequest,
}

impl Display for RequestError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            RequestError::InvalidHttpMethod(inner) => write!(f, "Invalid HTTP Method: {}", inner),
            RequestError::InvalidUri(inner) => write!(f, "Invalid URI: {}", inner),
            RequestError::InvalidHttpVersion(inner) => write!(f, "Invalid HTTP Version: {}", inner),
            RequestError::UnsupportedHeader => write!(f, "Unsupported header."),
            RequestError::InvalidHeader => write!(f, "Invalid header."),
            RequestError::InvalidRequest => write!(f, "Invalid request."),
        }
    }
}

/// Errors associated with a HTTP Connection.
#[derive(Debug)]
pub enum ConnectionError {
    /// The request parsing has failed.
    ParseError(RequestError),
    /// Could not perform a stream operation successfully.
    StreamError(std::io::Error),
    /// Attempted to read or write on a closed connection.
    ConnectionClosed,
    /// Attempted to write on a stream when there was nothing to write.
    InvalidWrite,
}

impl Display for ConnectionError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            ConnectionError::ParseError(inner) => write!(f, "Parsing error: {}", inner),
            ConnectionError::StreamError(inner) => write!(f, "Stream error: {}", inner),
            ConnectionError::ConnectionClosed => write!(f, "Connection closed."),
            ConnectionError::InvalidWrite => write!(f, "Invalid write attempt."),
        }
    }
}

/// Errors pertaining to `HttpRoute`.
#[derive(Debug)]
pub enum RouteError {
    /// Handler for http routing path already exists.
    HandlerExist(String),
}

impl Display for RouteError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            RouteError::HandlerExist(p) => write!(f, "handler for {} already exists", p),
        }
    }
}

/// Errors pertaining to `HttpServer`.
#[derive(Debug)]
pub enum ServerError {
    /// Epoll operations failed.
    IOError(std::io::Error),
    /// Error from one of the connections.
    ConnectionError(ConnectionError),
    /// Server maximum capacity has been reached.
    ServerFull,
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            ServerError::IOError(inner) => write!(f, "IO error: {}", inner),
            ServerError::ConnectionError(inner) => write!(f, "Connection error: {}", inner),
            ServerError::ServerFull => write!(f, "Server is full."),
        }
    }
}

/// The Body associated with an HTTP Request or Response.
///
/// ## Examples
/// ```
/// extern crate micro_http;
/// use micro_http::Body;
/// let body = Body::new("This is a test body.".to_string());
/// assert_eq!(body.raw(), b"This is a test body.");
/// assert_eq!(body.len(), 20);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Body {
    /// Body of the HTTP message as bytes.
    pub body: Vec<u8>,
}

impl Body {
    /// Creates a new `Body` from a `String` input.
    pub fn new<T: Into<Vec<u8>>>(body: T) -> Self {
        Body { body: body.into() }
    }

    /// Returns the body as an `u8 slice`.
    pub fn raw(&self) -> &[u8] {
        self.body.as_slice()
    }

    /// Returns the length of the `Body`.
    pub fn len(&self) -> usize {
        self.body.len()
    }

    /// Checks if the body is empty, ie with zero length
    pub fn is_empty(&self) -> bool {
        self.body.len() == 0
    }
}

/// Supported HTTP Methods.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Method {
    /// GET Method.
    Get,
    /// PUT Method.
    Put,
    /// PATCH Method.
    Patch,
    /// Delete Method.
    Delete,
}

impl Method {
    /// Returns a `Method` object if the parsing of `bytes` is successful.
    ///
    /// The method is case sensitive. A call to try_from with the input b"get" will return
    /// an error, but when using the input b"GET", it returns Method::Get.
    ///
    /// # Errors
    /// Returns `RequestError` if the method specified by `bytes` is unsupported.
    pub fn try_from(bytes: &[u8]) -> Result<Self, RequestError> {
        match bytes {
            b"GET" => Ok(Method::Get),
            b"PUT" => Ok(Method::Put),
            b"PATCH" => Ok(Method::Patch),
            b"DELETE" => Ok(Method::Delete),
            _ => Err(RequestError::InvalidHttpMethod("Unsupported HTTP method.")),
        }
    }

    /// Returns an `u8 slice` corresponding to the Method.
    pub fn raw(self) -> &'static [u8] {
        match self {
            Method::Get => b"GET",
            Method::Put => b"PUT",
            Method::Patch => b"PATCH",
            Method::Delete => b"DELETE",
        }
    }

    /// Returns an &str corresponding to the Method.
    pub fn to_str(self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Put => "PUT",
            Method::Patch => "PATCH",
            Method::Delete => "DELETE",
        }
    }
}

/// Supported HTTP Versions.
///
/// # Examples
/// ```
/// extern crate micro_http;
/// use micro_http::Version;
/// let version = Version::try_from(b"HTTP/1.1");
/// assert!(version.is_ok());
///
/// let version = Version::try_from(b"http/1.1");
/// assert!(version.is_err());
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Version {
    /// HTTP/1.0
    Http10,
    /// HTTP/1.1
    Http11,
}

impl Version {
    /// HTTP Version as an `u8 slice`.
    pub fn raw(self) -> &'static [u8] {
        match self {
            Version::Http10 => b"HTTP/1.0",
            Version::Http11 => b"HTTP/1.1",
        }
    }

    /// Creates a new HTTP Version from an `u8 slice`.
    ///
    /// The supported versions are HTTP/1.0 and HTTP/1.1.
    /// The version is case sensitive and the accepted input is upper case.
    ///
    /// # Errors
    /// Returns a `RequestError` when the version is not supported.
    pub fn try_from(bytes: &[u8]) -> Result<Self, RequestError> {
        match bytes {
            b"HTTP/1.0" => Ok(Version::Http10),
            b"HTTP/1.1" => Ok(Version::Http11),
            _ => Err(RequestError::InvalidHttpVersion(
                "Unsupported HTTP version.",
            )),
        }
    }

    /// Returns the default HTTP version = HTTP/1.1.
    pub fn default() -> Self {
        Version::Http11
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl PartialEq for ConnectionError {
        fn eq(&self, other: &Self) -> bool {
            use self::ConnectionError::*;
            match (self, other) {
                (ParseError(_), ParseError(_)) => true,
                (ConnectionClosed, ConnectionClosed) => true,
                (StreamError(_), StreamError(_)) => true,
                (InvalidWrite, InvalidWrite) => true,
                _ => false,
            }
        }
    }

    #[test]
    fn test_version() {
        // Tests for raw()
        assert_eq!(Version::Http10.raw(), b"HTTP/1.0");
        assert_eq!(Version::Http11.raw(), b"HTTP/1.1");

        // Tests for try_from()
        assert_eq!(Version::try_from(b"HTTP/1.0").unwrap(), Version::Http10);
        assert_eq!(Version::try_from(b"HTTP/1.1").unwrap(), Version::Http11);
        assert_eq!(
            Version::try_from(b"HTTP/2.0").unwrap_err(),
            RequestError::InvalidHttpVersion("Unsupported HTTP version.")
        );

        // Test for default()
        assert_eq!(Version::default(), Version::Http11);
    }

    #[test]
    fn test_method() {
        // Test for raw
        assert_eq!(Method::Get.raw(), b"GET");
        assert_eq!(Method::Put.raw(), b"PUT");
        assert_eq!(Method::Patch.raw(), b"PATCH");
        assert_eq!(Method::Delete.raw(), b"DELETE");

        // Tests for try_from
        assert_eq!(Method::try_from(b"GET").unwrap(), Method::Get);
        assert_eq!(Method::try_from(b"PUT").unwrap(), Method::Put);
        assert_eq!(Method::try_from(b"PATCH").unwrap(), Method::Patch);
        assert_eq!(Method::try_from(b"DELETE").unwrap(), Method::Delete);
        assert_eq!(
            Method::try_from(b"POST").unwrap_err(),
            RequestError::InvalidHttpMethod("Unsupported HTTP method.")
        );
    }

    #[test]
    fn test_body() {
        let body = Body::new("".to_string());
        // Test for is_empty
        assert!(body.is_empty());
        let body = Body::new("This is a body.".to_string());
        // Test for len
        assert_eq!(body.len(), 15);
        // Test for raw
        assert_eq!(body.raw(), b"This is a body.");
    }

    #[test]
    fn test_display_request_error() {
        assert_eq!(
            format!("{}", RequestError::InvalidHttpMethod("test")),
            "Invalid HTTP Method: test"
        );
        assert_eq!(
            format!("{}", RequestError::InvalidUri("test")),
            "Invalid URI: test"
        );
        assert_eq!(
            format!("{}", RequestError::InvalidHttpVersion("test")),
            "Invalid HTTP Version: test"
        );
        assert_eq!(
            format!("{}", RequestError::InvalidHeader),
            "Invalid header."
        );
        assert_eq!(
            format!("{}", RequestError::UnsupportedHeader),
            "Unsupported header."
        );
        assert_eq!(
            format!("{}", RequestError::InvalidRequest),
            "Invalid request."
        );
    }

    #[test]
    fn test_display_connection_error() {
        assert_eq!(
            format!(
                "{}",
                ConnectionError::ParseError(RequestError::InvalidRequest)
            ),
            "Parsing error: Invalid request."
        );
        assert_eq!(
            format!(
                "{}",
                ConnectionError::StreamError(std::io::Error::from_raw_os_error(11))
            ),
            "Stream error: Resource temporarily unavailable (os error 11)"
        );
        assert_eq!(
            format!("{}", ConnectionError::ConnectionClosed),
            "Connection closed."
        );
        assert_eq!(
            format!("{}", ConnectionError::InvalidWrite),
            "Invalid write attempt."
        );
    }

    #[test]
    fn test_display_server_error() {
        assert_eq!(
            format!(
                "{}",
                ServerError::ConnectionError(ConnectionError::ConnectionClosed)
            ),
            "Connection error: Connection closed."
        );
        assert_eq!(format!("{}", ServerError::ServerFull), "Server is full.");
        assert_eq!(
            format!(
                "{}",
                ServerError::IOError(std::io::Error::from_raw_os_error(11))
            ),
            "IO error: Resource temporarily unavailable (os error 11)"
        );
    }

    #[test]
    fn test_method_to_str() {
        let val = Method::Get;
        assert_eq!(val.to_str(), "GET");

        let val = Method::Put;
        assert_eq!(val.to_str(), "PUT");

        let val = Method::Patch;
        assert_eq!(val.to_str(), "PATCH");

        let val = Method::Delete;
        assert_eq!(val.to_str(), "DELETE");
    }
}
