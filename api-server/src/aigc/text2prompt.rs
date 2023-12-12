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
const SYSTEM_PROMPT: &str = include_str!("./prompts/test_sd_bot_2.txt");

pub async fn request(params: &serde_json::Value) -> GenerationParams {
    let prompt = params["prompt"].as_str().unwrap();
    let prompt = &format!(
        // "{} in the style of rendered in cinema4d, rococo still-lifes",
        // "{} ethereal fantasy concept art. magnificent, celestial, ethereal, painterly, epic, majestic, magical, fantasy art, cover art, dreamy.",
        "{}",
        prompt);
    let message = super::openai::request(&SYSTEM_PROMPT, prompt).await.unwrap();
    GenerationParams {
        prompt: message,
        negative_prompt: String::from(""),
    }
}
