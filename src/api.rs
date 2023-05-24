use log::debug;
use serde::{Serialize, Deserialize};
use anyhow::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct History {
    internal: Vec<Vec<String>>,
    visible: Vec<Vec<String>>
}

#[derive(Deserialize, Debug)]
pub struct Results {
    history: History
}

#[derive(Deserialize, Debug)]
pub struct Response {
    results: Vec<Results>,
}

#[derive(Serialize, Debug)]
pub struct Request<'a> {
    pub user_input: &'a str,
    pub history: History,
    pub mode: &'a str,  // Valid options: chat, chat-instruct, instruct
    pub character: &'a str,
    pub instruction_template: &'a str,

    pub regenerate: bool,
    pub _continue: bool,
    pub stop_at_newline: bool,
    pub chat_prompt_size: i32,
    pub chat_generation_attempts: i32,
    pub instruct_command: &'a str,

    pub max_new_tokens: i32,
    pub do_sample: bool,
    pub temperature: f32,
    pub top_p: f32,
    pub typical_p: i32,
    pub epsilon_cutoff: i32, // # In units of 1e-4
    pub eta_cutoff: i32,//  # In units of 1e-4
    pub repetition_penalty: f32,
    pub top_k: i32,
    pub min_length: i32,
    pub no_repeat_ngram_size: i32,
    pub num_beams: i32,
    pub penalty_alpha: i32,
    pub length_penalty: i32,
    pub early_stopping: bool,
    pub mirostat_mode: i32,
    pub mirostat_tau: i32,
    pub mirostat_eta: f32,
    pub seed: i32,
    pub add_bos_token: bool,
    pub truncation_length: i32,
    pub ban_eos_token: bool,
    pub skip_special_tokens: bool,
    pub stopping_strings: Vec<&'a str>
}

impl Default for Request<'_> {
    fn default() -> Self { 
        Self {
        user_input: "Hello",
        history: History { internal: vec![], visible: vec![] },
        mode: "instruct",  // Valid options: chat, chat-instruct, instruct
        character: "None",
        instruction_template: "Alpaca",

        regenerate: false,
        _continue: false,
        stop_at_newline: false,
        chat_prompt_size: 2048,
        chat_generation_attempts: 1,
        instruct_command: "Below is an instruction that describes a task. Write a response that appropriately completes the request.",
        // the above shouldnt matter as we're using an instruction template

        max_new_tokens: 2000,
        temperature: 0.7,
        top_k: 40,
        top_p: 0.1,
        typical_p: 1,
        repetition_penalty: 1.18,

        do_sample: true,
        epsilon_cutoff: 0, // # In units of 1e-4
        eta_cutoff: 0,//  # In units of 1e-4
        min_length: 0,
        
        no_repeat_ngram_size: 0,
        penalty_alpha: 0,
        num_beams: 1,
        length_penalty: 1,
        early_stopping: false,

        mirostat_mode: 0,
        mirostat_tau: 5,
        mirostat_eta: 0.1,

        seed: -1,
        add_bos_token: true,
        truncation_length: 2048,
        ban_eos_token: false,
        skip_special_tokens: true,
        stopping_strings: vec![]
        }
     }
}

pub async fn send_request(url: &str, request: &Request<'_>) -> Result<String> {
    debug!("Sending to url {url} request {request:?}");

    let client = reqwest::Client::new();
    let res = client.post(url)
        .json(request)
        .send()
        .await?
        .json::<Response>()
        .await?;
    debug!("Received response {res:?}");

    let text = res.results[0].history.visible.last().unwrap().last().unwrap();
    debug!("Received text {text}");

    Ok(text.to_string())
}