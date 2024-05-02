use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct GenerationParams {
    pub positive: String,
    pub negative: String,
}

#[derive(Deserialize)]
pub struct PromptResult {
    #[serde(rename = "Theme")]
    pub theme: String,
    #[serde(rename = "Kind")]
    pub kind: String,
    #[serde(rename = "Prompt")]
    pub prompt: String,
    #[serde(rename = "NegativePrompt")]
    pub negative_prompt: String,
}

const SYSTEM_PROMPT: &str = include_str!("./prompts/prompt_magic.txt");

pub async fn request(params: &serde_json::Value)
    -> Result<(GenerationParams, String), super::openai::OpenAIError>
{
    let mut user_input = params["prompt"].as_str().unwrap_or_default().to_owned();

    if user_input.contains("猪") {
        user_input = user_input.replace("猪", " pork ");
    }
    if user_input.contains("鸡") {
        user_input = user_input.replace("鸡", " chicken ");
    }
    if user_input.contains("胸") {
        user_input = user_input.replace("胸", " chest ");
    }

    let message_str = super::openai::request(
        "gpt-4", &SYSTEM_PROMPT, &user_input, 0.0, true
    ).await?;

    let prompt_result = serde_json::from_str::<PromptResult>(&message_str).map_err(|e| {
        tracing::warn!(user_input=user_input, "text2prompt result {}", message_str);
        super::openai::OpenAIError::Error(format!("Failed to parse text2prompt json: {:?}", e))
    })?;

    tracing::info!(user_input=user_input, "text2prompt result {}", message_str);

    let theme = prompt_result.theme;
    let kind = prompt_result.kind;
    let positive_prompt = prompt_result.prompt;
    // let negative_prompt = "((animal)), ((chicken)), ((logo)), human, hand, fingers, nsfw";
    let negative_prompt = prompt_result.negative_prompt;
    let style_index = if kind == "汉堡" || kind == "鸡肉卷" || kind == "小食" { 1 } else { 0 };

    let (style_positive, style_negative) = get_style(style_index);
    let positive_prompt = style_positive.replace("{prompt}", &positive_prompt);
    let negative_prompt = style_negative.replace("{prompt}", &negative_prompt);

    let generation_params = GenerationParams {
        positive: positive_prompt,
        negative: negative_prompt,
    };
    Ok((generation_params, String::from(theme)))
}

fn get_style(index: usize) -> (&'static str, &'static str) {
    let styles: Vec<(&str, &str)> = vec![(
        "{prompt}, ((solo food)) in the middle of the picture, close-up shot, ((masterpiece)), ((best quality)), 8k",
        "{prompt}, human, any part of the human body, lowres, bad anatomy, cropped, worst quality, low quality, poorly drawn, ugly, deformities, nsfw"
    ), (
        "food photography style, {prompt}. appetizing, award-winning, culinary, ((solo food)) in the middle of the picture, close-up shot, ((masterpiece)), ((best quality)), 8k",
        "{prompt}, unappetizing, sloppy, unprofessional, noisy, blurry, human, any part of the human body, lowres, bad anatomy, cropped, worst quality, low quality, poorly drawn, ugly, deformities, nsfw"
    ), (
        "breathtaking {prompt}. award-winning, professional, highly detailed",
        "{prompt}, ugly, deformed, noisy, blurry, distorted, grainy"
    ), (
        "neonpunk {prompt}. vaporwave, neon, vibes, vibrant, stunningly beautiful, crisp, detailed, sleek, ultramodern, magenta highlights, high contrast, cinema",
        "{prompt}, painting, drawing, illustration, glitch, deformed, mutated, cross-eyed, ugly, disfigured"
    ), (
        "ethereal fantasy concept art of {prompt}. magnificent, celestial, ethereal, painterly, epic, majestic, magical, fantasy art, cover art, dreamy",
        "{prompt}, photographic, realistic, realism, 35mm film, dslr, cropped, frame, text, deformed, glitch, noise, noisy, off-center, deformed, cross-eyed, closed eyes, bad anatomy, ugly, disfigured, sloppy"
    )];
    return styles[index];
}

#[allow(dead_code)]
fn get_magic_prompt() -> &'static str {
    let magics: Vec<&str> = vec![
        "magic circles",
        "fluorescent mushroom forest",
        "colorful bubble",
        "water drops, wet clothes, beautiful detailed water, floating, dynamic angle",
        "beautiful detailed glow, detailed ice, beautiful detailed water, floating palaces,ice crystal texture wings, Iridescence and rainbow hair",
        "beautiful detailed glow, detailed ice, beautiful detailed water, floating palaces ,ice crystal texture wings, Iridescence and rainbow hair",
        "beautiful detailed glow, detailed ice, beautiful detailed water, floating palaces, ice crystal texture wings",
        "detailed beautiful snow forest with trees, snowflakes, floating",
        "crystals texture Hair, beautiful detailed glass hair, glass shaped texture hand, crystallize texture body, gem body,hands as clear as jewels,crystallization of clothes, crystals texture skin, sparkle, lens flare, light leaks, broken glass, detailed glass shaped clothes, beautiful detailed gemstone sky, gemstone sea, crystals texture flowers, detailed crystallized clothing",
        "beautiful detailed glow, flames of war, nuclear explosion behide",
        "breeze, flying splashes, flying petals, wind",
        "surrounded by heavy floating sand flow and floating sharp stones, ink, illustration, watercolor",
        "detailed light, lightning in hand, lightning surrounds, lightning chain",
        "sunlight, angel, dynamic angle, floating, wing, halo, floating white silk, Holy Light, silver stars",
        "beautiful detailed pampas grass field, open hakama, surrounded by floating sakura, yellow full moon, beautiful detailed dark midnight sky, messy white long hair",
        "beautiful and delicate water, the finest grass, very delicate light, nature, painting, water spray, breeze, flowers and grass meadow, near the water edge, sunset, starry sky in a circle, randomly distributed clouds, river, splashing water, falling petals",
        "detailed light, feather, leaves, nature, sunlight, river, forest, bloom",
        "floating and rainbow long hair,Iridescence and rainbow, beautiful detailed starry sky",
        "chain ring, chain storm, dark chain, wholeblack bloomer, darkside, night, deep dark, darkness, dark clouds, ruins, shadow, death garden",
        "beautiful detailed glow, floating ashes, beautiful and detailed explosion, red moon, fire, fire cloud, wings on fire, a cloudy sky, smoke of gunpowder, burning, black dress, dove of peace, floating cloud",
        "beautiful detailed glow, detailed ice, beautiful detailed water, magic circle, floating palaces",
        "water bloom, delicate glow,  breeze, long   Flowers meadow, sunset, less stars form a circle, randomly distributed clouds, rivers, willows with branches falling into the water",
        "colorful bubble, floating,detailed light",
        "rose, vine, cage, bandage, red rope, detail light, falling rose petals",
        "starry tornado, starry Nebula, beautiful detailed sky",
        "moon, starry sky, lighting particle, fog, snow, bloom",
        "beautiful detailed glow, detailed ice, beautiful detailed water, cold full moon, snowflake, floating cloud",
        "burning forest, spark, light leaks, burning sky, flame, flames burning around, flying sparks",
        "destroyed, explosion, buildings in disarray, The residual eaves DuanBi, cumulus, mouldy, floating, wind, Dead end machine, broken robot, Mechanical robot girl, in the rubble of a devastated city",
        "mecha clothes, robot girl, sliver bodysuit, sliver and broken body",
        "Extremely gorgeous metal style, Metal crown with ornate stripes, Various metals background, Sputtered molten iron, floating hair, Hair like melted metal, Clothes made of silver, Clothes with gold lace, flowing gold and silver, everything flowing and melt, flowing iron, flowing silver, lace flowing and melt",
        "mecha clothes, robot girl",
        "ink, bone, ribs, rose, black hair, blue eyes, greyscale, no shadow, simple background, bright skin",
        "gorgeous crystal armor, crystal wings, altocumulus, clear_sky, snow mountain, flowery flowers, flowery bubbles, cloud map plane, crystal, crystal poppies,Brilliant light, thick_coating, glass tint, watercolor",
        "an extremely delicate and beautiful, floating, detailed wet clothes, detailed light, feather, nature, sunlight, river, floating palace, beautiful and delicate water, bloom, shine",
        "blue spark, red and blue hair, blue eyes, burning sky, flame, Rainbow in the sky, Flames burning ice, fire  butterflies, ice crystal texture wings, Flying sparks, detailed ice, a lot of luminous ice crystals, burning feathers, feathers made of ice, frozen feathers, ice and fire together",
        "anger, dragon horns, silver armor, metal, complex pattern, cape, indifference",
        "full body, helpless, tear, crying, falling from the sky, Weathering With You, falling, face towards the sky, hair flows upwards, disheveled hair, 1 girl, floating, beautiful detailed sky",
        "underwater, beautiful detailed water, coral, dynamic angle, floating, detailed light, floating hair, splash, fishes, leaves dress, feather, nature, sunlight, underwater forest, bloom, detailed glow, drenched, seaweed, fish, Tyndall effect",
        "extremely detailed CG unity 8k wallpaper, masterpiece, best quality, ultra-detailed, best illustration, best shadow, an extremely delicate and beautiful, dynamic angle,floating, fairyland,dynamic angle,sea of flowers,beautiful detailed garden,wind,classic,spring, detailed light, feather, nature, sunlight, river, forest, floating palace, the best building,beautiful and delicate water, painting, sketch, bloom, shine",
        "masterpiece, the best quality, super fine illustrations, beautiful and delicate water, very delicate light, nature, painting, fine lighting, more transparent stars, high-quality snowflakes, high-quality mountains, very fine 8KCG wallpapers, plateau, snow mountain, sunrise, randomly distributed clouds, snow field, cliff, rotating star sky, lake in mountain stream, luminous particles",
        "1980s style, simple background, retro artstyle",
        "white hair, red long hair, red eyes, full body, with sword, angry face, beautiful detailed eyes, Blood drop,Blood fog, floating hair, disheveled hair,  Splashing blood, Bloodstain",
        "dragon, dragon background",
        "hair fluttering in the wind, mechanical arm armor, mechanical body armor,riding motor, bodysuit, ruins of city in war, fire, burning cars, burning buildings, air force fleet in the sky",
        "mecha clothes, robot girl, sliver bodysuit, dragon wings, a dragon  stands behind the girl, beautiful detailed sliver dragon armor",
        "Beautiful butterflies in detail, Beautiful stars in detail, halter dress, particle, Starry sky in beautiful detail, Hazy fog, Ruins of beautiful details, Standing on the surface of the sea",
        "blonde wavy hair, shiny long hair, Gothic Lolita, blue white skirt, short skirt, black Headdress, bowknot, hair ornament, hair flower, Lace, cross-laced footwear, ribbon-trimmed sleeves, building architecture, gothic architecture, starry sky, outdoors, church, castle",
        "walking, waves, wind, glistening light of waves, detailed sunset glow, floating flow, coral, Luminous, coast, floating colorful bubbles, beautiful detailed sky, fluorescence,detailed shadow, conch, beautiful detailed water, starfish, meteor, rainbow, seabirds, glinting stars, glowworm, splash, detailed cloud, shell, fireworks",
        "beautiful detailed sky, night, stars, red plum blossom, winter, snowflakes, red and white flowers, starry sky, sitting, colorful, scenery, lantern, starfall",
    ];
    let mut rng = rand::thread_rng();
    let magic = magics.choose(&mut rng).unwrap();
    magic
}
