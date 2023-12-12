use serde_json::json;
use futures::future::join_all;
use std::env;

pub enum ComfyError {
    ReqwestError(reqwest::Error),
    SerdeJsonError(serde_json::Error),
}

async fn fetch_images(images_urls: Vec<String>) -> Vec<String> {
    async fn fetch(image_url: &str) -> String {
        // fetch image file from image_url and convert to base64
        let response = reqwest::get(image_url).await.unwrap();
        let image_bytes = response.bytes().await.unwrap();
        // println!("image_url: {:?}", image_url);
        return data_encoding::BASE64.encode(&image_bytes);
    }
    let base64_images = images_urls
        .iter()
        .map(|url| fetch(url))
        .collect::<Vec<_>>();
    return join_all(base64_images).await;
}

#[allow(dead_code)]
pub async fn request(prompt: &str) -> Result<Vec<String>, ComfyError> {
    let comfy_origin = env::var("COMFYUI_TEST_ORIGIN").unwrap();
    // let mut params: serde_json::Value = serde_json::from_str(COMFY_API_TPL_SDXL_TURBO).unwrap();
    // params["6"]["inputs"]["text"] = json!(prompt);
    let mut params: serde_json::Value = serde_json::from_str(COMFY_API_TPL_SDXL).unwrap();
    params["22"]["inputs"]["positive"] = json!(prompt);
    let payload = json!({
        "prompt": params,
    });
    let client = reqwest::Client::new();
    let url = format!("{}/prompt", comfy_origin);
    let res = match client.post(url).json(&payload).send().await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Error: {:?}", e);
            return Err(ComfyError::ReqwestError(e));
        }
    };
    let res_body_text = res.text().await.unwrap();
    // println!("body: {:?}", &res_body_text);
    let res_body_json: serde_json::Value = match serde_json::from_str(&res_body_text) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Error: {:?}", e);
            return Err(ComfyError::SerdeJsonError(e));
        }
    };
    let prompt_id = res_body_json["prompt_id"].as_str().unwrap();
    let base64_images: Vec<String>;
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let url = format!("{}/history/{}", comfy_origin, prompt_id);
        let res = match client.get(&url).send().await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Error: {:?}", e);
                return Err(ComfyError::ReqwestError(e));
            }
        };
        let res_body_text = res.text().await.unwrap();
        let res_body_json: serde_json::Value = serde_json::from_str(&res_body_text).unwrap();
        if let Some(result) = res_body_json.get(prompt_id) {
            // println!("result: {:?}", result["outputs"]["27"]["images"]);
            // let images_urls = result["outputs"]["27"]["images"]
            let images_urls = result["outputs"]["41"]["images"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    let filename = v["filename"].as_str().unwrap();
                    format!("{}/view?filename={}&subfolder=&type=output", comfy_origin, filename)
                })
                .collect();
            base64_images = fetch_images(images_urls).await;
            break;
        }
    }
    return Ok(base64_images);
}

#[allow(dead_code)]
const COMFY_API_TPL_SDXL_TURBO: &'static str = r#"
{"5":{"inputs":{"width":512,"height":512,"batch_size":4},"class_type":"EmptyLatentImage"},"6":{"inputs":{"text":"An innovative and visually appealing dish of chicken popcorn coated with a black Oreo-style crumb mixture. The chicken pieces are crispy and golden on the inside, with a unique black coating on the outside that resembles crushed Oreo cookies. This creates a striking contrast in colors and an intriguing blend of flavors. The chicken popcorn is arranged attractively in a bowl, inviting viewers to experience this unusual and delicious fusion of sweet and savory.","clip":["20",1]},"class_type":"CLIPTextEncode"},"7":{"inputs":{"text":"text, watermark","clip":["20",1]},"class_type":"CLIPTextEncode"},"8":{"inputs":{"samples":["13",0],"vae":["20",2]},"class_type":"VAEDecode"},"13":{"inputs":{"add_noise":true,"noise_seed":0,"cfg":1,"model":["20",0],"positive":["6",0],"negative":["7",0],"sampler":["14",0],"sigmas":["22",0],"latent_image":["5",0]},"class_type":"SamplerCustom"},"14":{"inputs":{"sampler_name":"euler_ancestral"},"class_type":"KSamplerSelect"},"20":{"inputs":{"ckpt_name":"sd_xl_turbo_1.0_fp16.safetensors"},"class_type":"CheckpointLoaderSimple"},"22":{"inputs":{"steps":1,"model":["20",0]},"class_type":"SDTurboScheduler"},"25":{"inputs":{"images":["8",0]},"class_type":"PreviewImage"},"27":{"inputs":{"filename_prefix":"ComfyUI","images":["8",0]},"class_type":"SaveImage"}}
"#;

#[allow(dead_code)]
const COMFY_API_TPL_SDXL: &'static str = r#"
{
  "22": {
    "inputs": {
      "base_ckpt_name": "sd_xl_base_1.0_0.9vae.safetensors",
      "base_clip_skip": -2,
      "refiner_ckpt_name": "sd_xl_refiner_1.0_0.9vae.safetensors",
      "refiner_clip_skip": -2,
      "positive_ascore": 6,
      "negative_ascore": 2,
      "vae_name": "Baked VAE",
      "positive": "a cyberpunk pig",
      "negative": "chicken, logo, human, drawing, painting, vector, graphics, longbody, lowres, bad anatomy, bad hands, missing fingers, extra digit, fewer digits, cropped, worst quality, low quality, poorly drawn, ugly, deformities, extra limbs, nsfw",
      "token_normalization": "none",
      "weight_interpretation": "comfy",
      "empty_latent_width": 1024,
      "empty_latent_height": 1024,
      "batch_size": 4
    },
    "class_type": "Eff. Loader SDXL"
  },
  "36": {
    "inputs": {
      "images": [
        "40",
        0
      ]
    },
    "class_type": "PreviewImage"
  },
  "39": {
    "inputs": {
      "noise_seed": 0,
      "steps": 20,
      "cfg": 7,
      "sampler_name": "dpmpp_2m_sde_gpu",
      "scheduler": "karras",
      "start_at_step": 0,
      "refine_at_step": -1,
      "preview_method": "auto",
      "vae_decode": "true",
      "sdxl_tuple": [
        "22",
        0
      ],
      "latent_image": [
        "22",
        1
      ]
    },
    "class_type": "KSampler SDXL (Eff.)"
  },
  "40": {
    "inputs": {
      "samples": [
        "39",
        1
      ],
      "vae": [
        "22",
        2
      ]
    },
    "class_type": "VAEDecode"
  },
  "41": {
    "inputs": {
      "filename_prefix": "ComfyUI",
      "images": [
        "40",
        0
      ]
    },
    "class_type": "SaveImage"
  }
}
"#;