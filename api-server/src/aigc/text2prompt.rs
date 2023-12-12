use serde::Serialize;

#[derive(Serialize)]
pub struct GenerationParams {
    pub prompt: String,
    pub negative_prompt: String,
}

// const _0_SYSTEM_PROMPT: &str = include_str!("./prompts/test_sd_bot_0.txt");
// const _1_SYSTEM_PROMPT: &str = include_str!("./prompts/test_sd_bot_1.txt");
// const _2_SYSTEM_PROMPT: &str = include_str!("./prompts/test_sd_bot_2.txt");
// const SYSTEM_PROMPT: &'static str = include_str!("./prompts/example_workflow_prompt.txt");
const SYSTEM_PROMPT: &str = include_str!("./prompts/workflow_prompt.txt");

pub async fn request(params: &serde_json::Value) -> GenerationParams {
    let user_input = params["prompt"].as_str().unwrap();
    let message_str = super::openai::request(
        &SYSTEM_PROMPT, user_input, 0.0).await.unwrap();
    tracing::info!(r#"text2prompt "{}" {}"#, user_input, message_str);

    let message_json: serde_json::Value = serde_json::from_str(&message_str).unwrap();
    let generation_prompt = message_json["Art Bot"].as_str().unwrap();

    let prefix = "food photography style, appetizing, scrumptious, professional, culinary, high-resolution, commercial, ";
    let suffix = ", (((solo))) food in the middle of the picture, close-up shot, ((masterpiece)), ((best quality)), 8k, highly detailed, ultra-detailed";
    let generation_prompt = format!("{}{}{}", prefix, generation_prompt, suffix);

    GenerationParams {
        prompt: generation_prompt,
        negative_prompt: String::from(""),
    }
}
