use std::{collections::HashMap, sync::{Arc, Mutex}, net::TcpStream, io::{BufReader, BufRead, Read}};

/// Struct representing a request to a server endpoint.
/// This is used internally by Iris but can be used to inspect the request at lower levels.
#[derive(Debug, Clone, Default)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: Vec<u8>,

    #[doc(hidden)]
    pub(crate) stream: Option<Arc<Mutex<TcpStream>>>,
}

macro_rules! read_line {
    ($buf:ident) => {{
        let mut line = String::new();
        $buf.read_line(&mut line).unwrap();
        line
    }};
}

impl Request {
    #[doc(hidden)]
    pub(crate) fn from_stream(stream: TcpStream) -> Self {
        let mut request = Request::default();

        let mut buf_reader = BufReader::new(&stream);

        // Parse the first line
        let first_line = read_line!(buf_reader);
        let first_line_split: Vec<_> = first_line.split(' ').collect();
        request.method = first_line_split[0].to_string();
        request.path = first_line_split[1].to_string();
        request.version = first_line_split[2].trim().to_string();

        // Parse the query params
        if request.path.contains('?') {
            let path_split: Vec<_> = request.path.split('?').map(|s| s.to_string()).collect();
            request.path = path_split[0].to_string();

            let query_params = path_split[1].clone();
            let query_params_split: Vec<_> = query_params.split('&').collect();
            for query_param in query_params_split {
                let query_param_split: Vec<_> = query_param.split('=').collect();
                request.query_params.insert(
                    query_param_split[0].to_string(),
                    query_param_split[1].to_string(),
                );
            }
        }

        // Ensure no / at the end of the path
        if request.path.ends_with('/') {
            request.path.pop();
        }

        // Parse the headers
        loop {
            let line = read_line!(buf_reader);
            if line.trim().is_empty() {
                break;
            }

            let line_split: Vec<_> = line.split(':').collect();
            request
                .headers
                .insert(line_split[0].to_string(), line_split[1].trim().to_string());
        }

        // Parse the body from the stream
        let content_length = request
            .headers
            .get("Content-Length")
            .unwrap_or(&"0".to_string())
            .parse::<usize>()
            .unwrap_or(0);

        let mut body = vec![0; content_length];
        buf_reader.read_exact(&mut body).unwrap();
        request.body = body;

        // Read the body from the buf_iter
        #[cfg(debug_assertions)]
        println!("{request:#?}");

        request.stream = Some(Arc::new(Mutex::new(stream)));
        request
    }

    /// Gets header value from the request by name and converts it to the specified type.
    /// If the header is not found None is returned.
    pub fn header<T: std::str::FromStr>(&self, name: &str) -> Option<T> {
        self.headers
            .get(name)
            .map(|s| match s.parse::<T>() {
                Ok(v) => v,
                Err(_) => panic!("Failed to parse header value"),
            })
    }
}