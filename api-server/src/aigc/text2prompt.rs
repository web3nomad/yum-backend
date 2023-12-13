use serde::Serialize;

#[derive(Serialize)]
pub struct GenerationParams {
    pub prompt: String,
    pub negative_prompt: String,
}

const SYSTEM_PROMPT: &str = include_str!("./prompts/workflow_prompt_art.txt");

pub async fn request(params: &serde_json::Value) -> Result<GenerationParams, super::openai::OpenAIError> {
    let user_input = params["prompt"].as_str().unwrap();
    let message_str = match super::openai::request(
        &SYSTEM_PROMPT, user_input, 0.0, true
    ).await {
        Ok(v) => v,
        Err(e) => {
            return Err(e);
        }
    };
    tracing::info!(r#"text2prompt "{}" {}"#, user_input, message_str);

    let message_json: serde_json::Value = serde_json::from_str(&message_str).unwrap();
    let generation_prompt = message_json["Art Bot"].as_str().unwrap();

    let prefix = "";
    let suffix = "";
    let generation_prompt = format!("{}{}{}", prefix, generation_prompt, suffix);

    let generation_params = GenerationParams {
        prompt: generation_prompt,
        negative_prompt: String::from(""),
    };
    Ok(generation_params)
}
