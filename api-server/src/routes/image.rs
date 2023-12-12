use serde_json::json;
use serde::Serialize;
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc;
use axum::{
    routing::{post, get},
    response::{Json, IntoResponse, Response},
    Router,
    http::StatusCode,
};
use diesel::prelude::*;
use crate::schema::tasks;
use crate::models::NewTask;
// use crate::models::Task;

fn establish_connection() -> MysqlConnection {
    let database_url = env::var("DATABASE_URL").unwrap();
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[derive(Serialize)]
struct GenerationParams {
    prompt: String,
    negative_prompt: String,
}

struct TaskPayload {
    task_id: String,
    params: serde_json::Value,
    generation_params: GenerationParams,
    callback_url: String,
}

struct BadRequest {
    message: String,
}

impl IntoResponse for BadRequest {
    fn into_response(self) -> Response {
        // let body = format!("Bad request: {}", self.message);
        let body = Json(json!({
            "error": self.message
        }));
        let status = StatusCode::BAD_REQUEST;
        (status, body).into_response()
    }
}

const _0_SYSTEM_PROMPT: &str = r#"
你是一个 KFC 的美食专家, 擅长编写 Stable Diffusion 的 prompt 来生成 KFC 的食物图片.
我将提供一些灵感来源, 口味, 和食物的类型, 你的任务是:
1. 将用户输入的内容翻译成英文, 下一步使用翻译后的结果;
2. 生成一段可以生成创意 KFC 食物的 Stable Diffusion prompt, 要求如下:
  - prompt 使用英文, 且不超过 500 个字符
  - prompt 中控制食物在图片中间, 使用近景拍摄视角, 背景要抽象
  - prompt 中必须保留用户输入的内容, 并加强权重, 用 (( 和 )) 包裹
  - prompt 中不要出现 KFC 这个单词
3. 最后直接输出第 2 步的 prompt, 不要包含第 1 步的结果，且不包含任何说明和解释.

如下是一个输入示例:
酥脆的,酸奶,汉堡
其中灵感来源是"酥脆的", 口味是"酸奶", 食物类型是"汉堡"

如下是一个优秀文案的示例:
Food photography style. nuggets coated with a black Oreo-style crumb mixture. Appetizing, professional, culinary, high-resolution, commercial, highly detailed. In the style of rendered in cinema4d, rococo still-lifes.
"#;

const _1_SYSTEM_PROMPT: &str = r#"
请依次执行以下两个任务，其中 Task 1 的结果作为 Task 2 的输入，最后只输出 Task 2 的结果。
---
# Task 1:
XD Bot 是一个美食文案的翻译专家，可以将自然语言翻译成“灵感,食材,食物”格式的英文文本，使用“,”隔开，第一部分是食物的灵感来源，第二部分是口味或者食材，第三部分是食物的名称，其中最后食物名称应尽量对应到下面的食物列表中：
- KFC Original Recipe Chicken
- KFC Extra Crispy Chicken
- KFC Kentucky Grilled Chicken
- KFC Nashville Hot Chicken
- Chicken Nuggets
- Popcorn Nuggets
- Chicken Little sandwiches
- Chicken Longer Burger
- Spicy Chicken Sandwich
- Chicken buckets
- Chicken drumsticks
- Chicken burgers
- Potato wedges
- Mashed potatoes
- Green beans
- Mac and cheese
- Sweet kernel corns
- Drinks
- Lime chili wings
- French fries
- KFC Rice
- Egg Tart
---
# Task 2:
Ginko Bot 是一位有艺术气质的 AI 助理，帮助人通过将自然语言转化为食物摄影的 prompt。Ginko Bot 的行动规则如下：
1. 第一部分：Food photography style, appetizing, scrumptious, professional, culinary, high-resolution, commercial,
2. 第二部分：用单词或者词组描述画面的所有主体元素，元素之间用“,"隔开，请给这些主体元素的英文词组增加小括号，在最后一个单词增加3个小括号，如：((clouds)), ((ham)), (((hamburger))), 输出内容；如果用户的第二个单词输入的是动物，比如“bullfrog”，则在第二个单词后面增加 transparent rubber made cute toy as the size of a nail 变为 bullfrog silver made ((tiny cute toy)), 输出内容
3. 第三部分：提供详细的、有创意的画面主体元素描述，以激发 AI 独特而有趣的图像，您可以描述未来城市的场景，或者充满奇怪生物的超现实景观。您的描述越详细、越富有想象力，生成的图像就会越有趣。如：A whimsical scene of fluffy clouds shaped like giant hamburger with ham flavor, floating in a bright blue sky, casting playful shadows on the ground below，输出内容
4. 第四部分：(((solo))) food in the middle of the picture, close-up shot, ((masterpiece)), ((best quality)), 8k, highly detailed, ultra-detailed
5. Ginko Bot 会将以上生成的四部分文本用逗号连接，中间不包含任何换行符的 prompt 作为最终结果；
6. Ginko Bot 输出时将直接输出 prompt，而不包含任何说明和解释。生成结果为英文，且只包含英文单数名词。
"#;

const SYSTEM_PROMPT: &str = r#"
Ginko Bot 是一位有艺术气质的 AI 助理，帮助人通过将自然语言转化为食物摄影的 prompt。Ginko Bot 的行动规则如下：
1. 第一部分：Food photography style, appetizing, scrumptious, professional, culinary, high-resolution, commercial, 输出内容；
2. 第二部分：用单词或者词组描述画面的所有主体元素，元素之间用“,"隔开，请给这些主体元素的英文词组增加小括号，在最后一个单词增加3个小括号，如：((clouds)), ((ham)), (((hamburger))), 第一个单词要翻译成抽象的概念，第二个单词应扩写为一种食物的口味或者正常的食材，第三个单词应尽量对应 KFC 餐厅里的食物，输出内容；
3. 第三部分：提供详细的、有创意的画面主体元素描述，以激发 AI 独特而有趣的图像，您可以描述未来城市的场景，或者充满奇怪生物的超现实景观。您的描述越详细、越富有想象力，生成的图像就会越有趣，输出内容；
4. 第四部分：(((solo))) food in the middle of the picture, close-up shot, ((masterpiece)), ((best quality)), 8k, highly detailed, ultra-detailed, 输出内容；
5. Ginko Bot 会将以上生成的四部分文本用逗号连接，中间不包含任何换行符的 prompt 作为最终结果；
6. Ginko Bot 输出时将直接输出 prompt，而不包含任何说明和解释。生成结果为英文，且只包含英文单数名词。
"#;

async fn get_generation_params(params: &serde_json::Value) -> GenerationParams {
    let prompt = params["prompt"].as_str().unwrap();
    let prompt = &format!(
        // "{} in the style of rendered in cinema4d, rococo still-lifes",
        // "{} ethereal fantasy concept art. magnificent, celestial, ethereal, painterly, epic, majestic, magical, fantasy art, cover art, dreamy.",
        "{}",
        prompt);
    let message = crate::aigc::openai::request(&SYSTEM_PROMPT, prompt).await.unwrap();
    GenerationParams {
        prompt: message,
        negative_prompt: String::from(""),
    }
}

async fn handle_generate_image_request(
    body: String,
    tx: Arc<mpsc::Sender<TaskPayload>>
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let timestamp = chrono::Utc::now().timestamp_millis();
    let random = rand::random::<u32>() % 1000000;
    let task_id = format!("{}-{:0>6}", timestamp, random);

    let body_json = serde_json::from_str::<serde_json::Value>(&body).unwrap();
    let params = body_json["params"].to_owned();
    let callback_url = body_json["resultCallbackUrl"].as_str().unwrap().to_string();
    let generation_params = get_generation_params(&params).await;

    let task_payload = TaskPayload {
        task_id: task_id.clone(),
        params,
        generation_params,
        callback_url,
    };

    let conn = &mut establish_connection();
    let new_task = NewTask {
        task_id: &task_id,
        params: &serde_json::to_string(&task_payload.params).unwrap(),
        generation_params: &serde_json::to_string(&task_payload.generation_params).unwrap(),
        result: "",
        callback_url: &task_payload.callback_url,
    };
    diesel::insert_into(tasks::table)
        .values(&new_task)
        .execute(conn)
        .expect("Error saving new task");

    if let Ok(()) = tx.try_send(task_payload) {
        let res_json = json!({
            "taskId": &task_id
        });
        return Ok(Json(res_json));
    } else {
        return Err(BadRequest { message: "Queue is full".to_string() });
    }
}

async fn callback_generate_image(task_payload: &TaskPayload, result: &serde_json::Value) {
    let callback_res = reqwest::Client::new()
        .post(&task_payload.callback_url)
        .json(&json!({
            "taskId": task_payload.task_id,
            "params": task_payload.params,
            "result": result,
            "generation_params": task_payload.generation_params,
        }))
        .send().await;
    match callback_res {
        Ok(_) => tracing::info!("Task {} callback success", task_payload.task_id),
        Err(e) => tracing::error!("Task {} callback failed: {}", task_payload.task_id, e)
    }
}

async fn process_task(task_payload: &TaskPayload) {
    let conn = &mut establish_connection();
    let task_id = &task_payload.task_id;
    tracing::info!("Task {} started", task_id);
    diesel::update(tasks::table)
        .filter(tasks::task_id.eq(task_id))
        .set(tasks::starts_at.eq(chrono::Utc::now().naive_utc()))
        .execute(conn).unwrap();
    let prompt = &task_payload.generation_params.prompt;

    if let Ok(base64_images) = crate::aigc::comfy::request(prompt).await {
        tracing::info!("Task {} comfy success", task_id);

        let task_id: &str = &task_payload.task_id;
        let images = base64_images
            .iter()
            .enumerate()
            .map(|(i, base64_image)| {
                let filename = format!("{}-{}.png", task_id, i);
                ( filename, base64_image )
            }).collect::<Vec<_>>();
        let image_urls = crate::storage::azure::upload_images(&images).await;

        let result = json!({
            "images": image_urls.iter().map(|image_url| {
                json!({
                    "src": image_url
                })
            }).collect::<Vec<_>>()
        });

        let conn = &mut establish_connection();
        diesel::update(tasks::table)
            .filter(tasks::task_id.eq(task_id))
            .set((
                tasks::result.eq(serde_json::to_string(&result).unwrap()),
                tasks::ends_at.eq(chrono::Utc::now().naive_utc())
            ))
            .execute(conn)
            .expect("Error updating task");

        callback_generate_image(&task_payload, &result).await;

        tracing::info!("Task {} end", task_id);
    } else {
        tracing::info!("Task {} comfy failed", task_id);
    }
}

pub fn get_routes() -> Router {
    let (
        tx,
        mut rx
    ) = mpsc::channel::<TaskPayload>(10);

    let tx = Arc::new(tx);

    tokio::spawn(async move {
        while let Some(task_payload) = rx.recv().await {
            process_task(&task_payload).await;
        }
    });

    let router: Router = Router::new()
        .route("/api/yum/generate/image", post({
            let tx = Arc::clone(&tx);
            |body: String| handle_generate_image_request(body, tx)
        }))
        .route("/api/yum/generate/queueInfo", get(|| async {
            let queue_info = json!({
                "pendingTasks": 5,
                "executingTasks": 1,
            });
            Json(queue_info)
        }));
    return router;
}
