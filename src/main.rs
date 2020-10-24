use clap::{App, Arg};
use native_tls::TlsConnector;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::time::Instant;
use url::Url;

struct Response {
    time_taken: u128,
    status: u16,
    size: usize,
    body: String,
}

fn main() {
    let matches = App::new("Dinesh's HTTP Profiler")
        .version("0.1")
        .author("Dinesh Kumar Ranganathan <dkr2@illinois.edu>")
        .about("HTTP Profiler submission for Cloudflare Systems Challenge")
        .arg(
            Arg::with_name("url")
                .long("url")
                .takes_value(true)
                .help("URL to profile")
                .required(true),
        )
        .arg(
            Arg::with_name("runs")
                .long("profile")
                .takes_value(true)
                .help("Number of requests to make when profiling input URL"),
        )
        .get_matches();
    let url_str = matches.value_of("url").expect("No URL entered");
    let runs_str = matches.value_of("runs");
    match runs_str {
        None => println!("{}", request(url_str, true).body),
        Some(s) => match s.parse::<usize>() {
            Ok(n) if n > 0 => profile(url_str, n),
            _ => panic!("Invalid profile number"),
        },
    }
}

fn request(url_str: &str, need_body: bool) -> Response {
    let url = Url::parse(url_str).expect("Invalid URL");
    let mut request_data = String::new();
    request_data.push_str("GET /");
    request_data.push_str(url.path());
    request_data.push_str(" HTTP/1.0");
    request_data.push_str("\r\n");
    request_data.push_str("Host: ");
    request_data.push_str(url.host_str().unwrap());
    request_data.push_str("\r\n");
    request_data.push_str("Connection: close");
    request_data.push_str("\r\n");
    request_data.push_str("\r\n");
    let connector = TlsConnector::new().unwrap();

    let now = Instant::now();
    let stream = TcpStream::connect(url.host_str().unwrap().to_owned() + ":443")
        .expect("Unable to establish stream");
    let mut stream = connector.connect(url.host_str().unwrap(), stream).unwrap();
    stream
        .write_all(request_data.as_bytes())
        .expect("Unable to write to stream");

    let mut buf = Vec::new();
    stream
        .read_to_end(&mut buf)
        .expect("Unable to read from stream");
    let request_time = now.elapsed().as_millis();
    let response = String::from_utf8_lossy(&buf);

    return parse_response(&response, request_time, need_body);
}

fn parse_response(response: &str, request_time: u128, need_body: bool) -> Response {
    let lines: Vec<&str> = response.lines().collect();
    let http_code: Vec<&str> = lines[0].split(" ").collect();
    let http_code = http_code[1].parse::<u16>().unwrap();
    let mut response_body: String = String::new();
    if need_body {
        response_body = extract_body(lines);
    }
    let response_struct = Response {
        time_taken: request_time,
        status: http_code,
        size: response.len(),
        body: response_body,
    };
    return response_struct;
}

fn extract_body(lines: Vec<&str>) -> String {
    let mut is_body = false;
    let mut response_body = String::new();

    for (index, line) in lines.iter().enumerate() {
        if is_body == true {
            response_body = lines[index..].join("\n");
            break;
        }
        if line.is_empty() {
            is_body = true;
        }
    }
    return response_body;
}

fn profile(url: &str, runs: usize) {
    let mut responses: Vec<Response> = Vec::new();
    for _ in 0..runs {
        responses.push(request(url, false));
    }
    let times: Vec<u128> = responses.iter().map(|x| x.time_taken).collect();
    let mut unsuccessful_codes: Vec<u16> = responses
        .iter()
        .filter(|x| x.status < 200 || x.status >= 300)
        .map(|x| x.status)
        .collect();
    println!("Number of requests: {}", runs);
    println!("Fastest response time: {} ms", times.iter().min().unwrap());
    println!("Slowest response time: {} ms", times.iter().max().unwrap());
    println!("Mean response time: {:.3} ms", mean(&times));
    println!("Median response time: {} ms", median(times));
    println!(
        "Percentage of successful requests: {}%",
        (runs - unsuccessful_codes.len()) as f32 / runs as f32 * 100.0
    );
    if !unsuccessful_codes.is_empty() {
        unsuccessful_codes.sort_unstable();
        unsuccessful_codes.dedup();
        let codes_str = unsuccessful_codes
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        println!("Unsuccessful Codes: {}", codes_str);
    }
    println!(
        "Smallest response: {} bytes",
        responses.iter().map(|x| x.size).min().unwrap()
    );
    println!(
        "Largest response: {} bytes",
        responses.iter().map(|x| x.size).max().unwrap()
    );
}

fn mean(numbers: &Vec<u128>) -> f64 {
    let sum: u128 = numbers.iter().sum();
    sum as f64 / numbers.len() as f64
}

fn median(mut numbers: Vec<u128>) -> u128 {
    numbers.sort_unstable();

    let mid = numbers.len() / 2;
    if numbers.len() % 2 == 0 {
        mean(&vec![numbers[mid - 1], numbers[mid]]) as u128
    } else {
        numbers[mid]
    }
}
