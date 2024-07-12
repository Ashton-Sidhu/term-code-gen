use clap::Parser;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use std::{env, io};

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Cli {
    /// Code to generate tests from
    code: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let code = match cli.code {
        Some(i) => i,
        None => {
            let mut buffer = String::new();
            let stdin = io::stdin();
            match stdin.read_line(&mut buffer) {
                Ok(_) => {
                    if buffer.is_empty() {
                        panic!(
                            "No code provided. Must provide an input via an arg or through stdin."
                        )
                    }
                }
                Err(_) => panic!("Error reading from stdin."),
            }
            buffer
        }
    };

    let openai_api_key = env::var("OPENAI_API_KEY").unwrap().to_string();
    let client = Client::new();
    let mut headers = HeaderMap::new();

    let auth_string = format!("Bearer {openai_api_key}");

    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        auth_string
            .try_into()
            .expect("Invalid Chars in bearer token."),
    );

    let identify_lang_req_json = json!({
        "model": "gpt-4o",
        "messages": [
            {
                "role": "system",
                "content": String::from("You are an AI Assistant specialized in identifying what language code is written in. Respond with only the coding language the text supplied is written in. If you do not know respond with: NA")
            },
            {
                "role": "user",
                "content": code
            }
        ]
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .headers(headers.clone())
        .json(&identify_lang_req_json)
        .send();

    let lang_result: Value = serde_json::from_str(&response.unwrap().text().unwrap()).unwrap();

    // let lang_result: Value = serde_json::from_str(&response.unwrap().text().unwrap());
    let language = lang_result["choices"][0]["message"]["content"]
        .as_str()
        .unwrap();

    let generate_test_req_json = json!({
        "model": "gpt-4o",
        "messages": [
            {
                "role": "system",
                "content": String::from("You are an AI assistant that generates tests for code provided by the user in their speicified language. Return the code as text and nothing else. Do not use Markdown.")
            },
            {
                "role": "user",
                "content": format!("Generate test functions in {language} for the following code: {cli_code}", cli_code=code)
            }
        ],
        "temperature": 0
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .headers(headers.clone())
        .json(&generate_test_req_json)
        .send();

    let lang_result: Value = serde_json::from_str(&response.unwrap().text().unwrap()).unwrap();

    let test_code = lang_result["choices"][0]["message"]["content"]
        .as_str()
        .unwrap();

    println!("{}", test_code);
}
