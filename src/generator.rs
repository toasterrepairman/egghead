use std::time::Duration;
use rust_bert::gpt_neo::{
    GptNeoConfigResources, GptNeoMergesResources, GptNeoModelResources, GptNeoVocabResources,
};
use rust_bert::pipelines::common::ModelType;
use rust_bert::pipelines::sentiment::{Sentiment, SentimentModel};
use rust_bert::pipelines::sequence_classification::{SequenceClassificationConfig, SequenceClassificationModel};
use rust_bert::pipelines::text_generation::{TextGenerationConfig, TextGenerationModel};
use rust_bert::reformer::{ReformerConfigResources, ReformerModelResources, ReformerVocabResources};
use rust_bert::resources::RemoteResource;
use rust_bert::roberta::{RobertaConfigResources, RobertaMergesResources, RobertaModelResources, RobertaVocabResources};
use serenity::json::Value;
use serenity::model::channel::Message;
use tch::Device;

pub fn ask(question: &str, context: &str) -> String {
    let config_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoConfigResources::GPT_NEO_125M,
    ));
    let vocab_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoVocabResources::GPT_NEO_125M,
    ));
    let merges_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoMergesResources::GPT_NEO_125M,
    ));
    let model_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoModelResources::GPT_NEO_125M,
    ));
    let generate_config = TextGenerationConfig {
        model_type: ModelType::GPTNeo,
        model_resource,
        config_resource,
        vocab_resource,
        merges_resource: merges_resource,
        min_length: 20,
        max_length: 150,
        early_stopping: true,
        do_sample: false,
        num_beams: 1,
        num_return_sequences: 1,
        repetition_penalty: 104.5,
        temperature: 3.4,
        diversity_penalty: Some(3.0),
        no_repeat_ngram_size: 3,
        device: Device::Cpu,
        ..Default::default()
    };

    let mut model = TextGenerationModel::new(generate_config)
        .expect("This regularly blows up");
    model.set_device(Device::cuda_if_available());

    let input_context_1 = format!("{}\n", question);
    let mut output = model.generate((&[&input_context_1]), None).pop();
    return output.unwrap().split_off(input_context_1.len())
}

pub fn wiki(question: &str, context: &str) -> String {
    let config_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoConfigResources::GPT_NEO_125M,
    ));
    let vocab_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoVocabResources::GPT_NEO_125M,
    ));
    let merges_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoMergesResources::GPT_NEO_125M,
    ));
    let model_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoModelResources::GPT_NEO_125M,
    ));
    let generate_config = TextGenerationConfig {
        model_type: ModelType::GPTNeo,
        model_resource,
        config_resource,
        vocab_resource,
        merges_resource: merges_resource,
        min_length: 20,
        max_length: 180,
        early_stopping: true,
        do_sample: false,
        num_beams: 1,
        num_return_sequences: 1,
        repetition_penalty: 104.5,
        temperature: 3.4,
        diversity_penalty: Some(3.0),
        no_repeat_ngram_size: 3,
        device: Device::Cpu,
        ..Default::default()
    };

    let mut model = TextGenerationModel::new(generate_config)
        .expect("This regularly blows up");
    model.set_device(Device::cuda_if_available());

    let input_context_1 = format!("{}", question);
    let mut output = model.generate((&[&input_context_1]), None).pop();
    return output.unwrap()
}

pub fn hn(question: &str, context: &str) -> String {
    let config_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoConfigResources::GPT_NEO_125M,
    ));
    let vocab_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoVocabResources::GPT_NEO_125M,
    ));
    let merges_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoMergesResources::GPT_NEO_125M,
    ));
    let model_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoModelResources::GPT_NEO_125M,
    ));
    let generate_config = TextGenerationConfig {
        model_type: ModelType::GPTNeo,
        model_resource,
        config_resource,
        vocab_resource,
        merges_resource: merges_resource,
        min_length: 20,
        max_length: 80,
        early_stopping: true,
        do_sample: false,
        num_beams: 1,
        num_return_sequences: 1,
        repetition_penalty: 104.5,
        temperature: 2.4,
        diversity_penalty: Some(9.0),
        no_repeat_ngram_size: 3,
        device: Device::Cpu,
        ..Default::default()
    };

    let mut model = TextGenerationModel::new(generate_config)
        .expect("This regularly blows up");
    model.set_device(Device::cuda_if_available());

    let input_context_1 = format!("{}", question);
    let mut output = model.generate((&[&input_context_1]), None).pop();
    return output.unwrap()
}

pub fn script(task: &str, context: &str) -> String {
    let config_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoConfigResources::GPT_NEO_125M,
    ));
    let vocab_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoVocabResources::GPT_NEO_125M,
    ));
    let merges_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoMergesResources::GPT_NEO_125M,
    ));
    let model_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoModelResources::GPT_NEO_125M,
    ));
    let generate_config = TextGenerationConfig {
        model_type: ModelType::GPTNeo,
        model_resource,
        config_resource,
        vocab_resource,
        merges_resource: merges_resource,
        min_length: 20,
        max_length: 150,
        early_stopping: true,
        do_sample: false,
        num_beams: 1,
        num_return_sequences: 1,
        repetition_penalty: 150.5,
        temperature: 2.1,
        diversity_penalty: Some(15.0),
        no_repeat_ngram_size: 1,
        device: Device::Cpu,
        ..Default::default()
    };

    let mut model = TextGenerationModel::new(generate_config)
        .expect("This regularly blows up");
    model.set_device(Device::cuda_if_available());

    let input_context_1 = format!("{}", task);
    let mut output = model.generate((&[&input_context_1]), None).pop();
    return output.unwrap()
}

pub fn analyze(context: &str) -> Vec<Sentiment> {
    //    Set-up classifier
    let sentiment_classifier = SentimentModel::new(Default::default()).unwrap();

    //    Define input
    let input = [
        context
    ];

    //    Run model
    return sentiment_classifier.predict(input);
}

use reqwest::Error;

#[derive(serde::Serialize)]
struct Request {
    text: String,
    topP: f32,
    topK: i32,
    temperature: f32,
    tokens: i32,
}

#[derive(serde::Deserialize, Debug)]
struct Response {
    prediction: String,
}

pub fn call_api(prompt: &str) -> Result<String, Error> {
    let request = Request {
        text: prompt.to_string(),
        topP: 0.8,
        topK: 50,
        temperature: 0.7,
        tokens: 100,
    };

    let client = reqwestClient::builder()
        .timeout(Duration::from_secs(60))
        .build()?;

    let res = client
        .post("http://localhost:8080/predict")
        .header("Content-Type", "application/json")
        .json(&request)
        .send()?
        .json::<Response>()?;

    Ok(res.prediction)

}
