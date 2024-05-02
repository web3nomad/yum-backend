pub fn modify_text(user_input: &str) -> String {
    let user_input = user_input.to_string();
    // if user_input.contains("猪") {
    //     user_input = user_input.replace("猪", " pork ");
    // }
    // if user_input.contains("鸡") {
    //     user_input = user_input.replace("鸡", " chicken ");
    // }
    // if user_input.contains("胸") {
    //     user_input = user_input.replace("胸", " chest ");
    // }
    // user_input
    format!("Food Inspiration: {}", user_input)
}
