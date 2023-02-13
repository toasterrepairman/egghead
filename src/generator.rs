use rust_bert::gpt_neo::{
    GptNeoConfigResources, GptNeoMergesResources, GptNeoModelResources, GptNeoVocabResources,
};
use rust_bert::pipelines::common::ModelType;
use rust_bert::pipelines::question_answering::{QaInput, QuestionAnsweringModel};
use rust_bert::pipelines::sentiment::SentimentModel;
use rust_bert::pipelines::sequence_classification::{SequenceClassificationConfig, SequenceClassificationModel};
use rust_bert::pipelines::text_generation::{TextGenerationConfig, TextGenerationModel};
use rust_bert::reformer::{ReformerConfigResources, ReformerModelResources, ReformerVocabResources};
use rust_bert::resources::RemoteResource;
use rust_bert::roberta::{RobertaConfigResources, RobertaMergesResources, RobertaModelResources, RobertaVocabResources};
use serenity::model::channel::Message;
use tch::Device;

pub(crate) const PROMPT: &str = "Q:";

pub fn stupid(question: &str, context: &str) -> String {
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
        merges_resource: Some(merges_resource),
        min_length: min_len,
        max_length: max_len,
        do_sample: false,
        early_stopping: true,
        num_beams: 4,
        num_return_sequences: 1,
        device: Device::Cpu,
        repetition_penalty: 10.0,
        temperature: 2.0,
        ..Default::default()
    };

    let mut model = TextGenerationModel::new(generate_config)
        .expect("This regularly blows up");
    model.set_device(Device::cuda_if_available());

    let input_context_1 = question;
    let mut output = model.generate((&[PROMPT, input_context_1]), None).pop();
    return output.unwrap()
}

pub fn smart(prompt: &str) -> String {
    let config_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoConfigResources::GPT_NEO_1_3B,
    ));
    let vocab_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoVocabResources::GPT_NEO_1_3B,
    ));
    let merges_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoMergesResources::GPT_NEO_1_3B,
    ));
    let model_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoModelResources::GPT_NEO_1_3B,
    ));
    let generate_config = TextGenerationConfig {
        model_type: ModelType::GPTNeo,
        model_resource,
        config_resource,
        vocab_resource,
        merges_resource: Some(merges_resource),
        min_length: 20,
        max_length: Some(180),
        do_sample: true,
        top_k: 50,
        early_stopping: true,
        num_beams: 3,
        num_return_sequences: 1,
        repetition_penalty: 18.0,
        length_penalty: 16.0,
        temperature: 1.70,
        device: Device::Cpu,
        ..Default::default()
    };

    let mut model = TextGenerationModel::new(generate_config)
        .expect("This regularly blows up");
    model.set_device(Device::cuda_if_available());

    let input_context_1 = prompt;
    let mut output = model.generate(&[PROMPT, input_context_1], None).pop();
    return output.unwrap()
}


pub fn code(prompt: &str) -> String {
    //    Language identification
    let sequence_classification_config = SequenceClassificationConfig::new(
        ModelType::Roberta,
        RemoteResource::from_pretrained(RobertaModelResources::CODEBERTA_LANGUAGE_ID),
        RemoteResource::from_pretrained(RobertaConfigResources::CODEBERTA_LANGUAGE_ID),
        RemoteResource::from_pretrained(RobertaVocabResources::CODEBERTA_LANGUAGE_ID),
        Some(RemoteResource::from_pretrained(
            RobertaMergesResources::CODEBERTA_LANGUAGE_ID,
        )),
        false,
        None,
        None,
    );

    let sequence_classification_model =
        SequenceClassificationModel::new(sequence_classification_config).expect("awkward");

    //    Define input
    let input = [prompt];

    let mut response = String::new();

    //    Run model
    let output = sequence_classification_model.predict(input);
    for label in output {
        response.push_str(&label.text)
    }
    return response;
}

pub fn analyze(context: String) {
    //    Set-up classifier
    let sentiment_classifier = SentimentModel::new(Default::default()).unwrap();

    //    Define input
    let input = [

    ];

    //    Run model
    let output = sentiment_classifier.predict(input);
}