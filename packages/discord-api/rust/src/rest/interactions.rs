#[derive(Debug)]
#[napi(object)]
pub struct NapiCreateInteractionResponseOptions {
    pub interaction_id: String,
    pub interaction_token: String,
}
