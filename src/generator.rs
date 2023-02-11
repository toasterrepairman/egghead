use rust_bert::gpt_neo::{
    GptNeoConfigResources, GptNeoMergesResources, GptNeoModelResources, GptNeoVocabResources,
};
use rust_bert::pipelines::common::ModelType;
use rust_bert::pipelines::question_answering::{QaInput, QuestionAnsweringModel};
use rust_bert::pipelines::sequence_classification::{SequenceClassificationConfig, SequenceClassificationModel};
use rust_bert::pipelines::text_generation::{TextGenerationConfig, TextGenerationModel};
use rust_bert::reformer::{ReformerConfigResources, ReformerModelResources, ReformerVocabResources};
use rust_bert::resources::RemoteResource;
use rust_bert::roberta::{RobertaConfigResources, RobertaMergesResources, RobertaModelResources, RobertaVocabResources};
use serenity::model::channel::Message;
use tch::Device;

pub(crate) const PROMPT: &str = "Format a response to this prompt in Markdown:\n";

pub fn generate(prompt: &str, min_len: i64, max_len: Option<i64>) -> String {
    //    Set-up model resources
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
        do_sample: true,
        early_stopping: false,
        num_beams: 5,
        num_return_sequences: 1,
        device: Device::Cpu,
        repetition_penalty: 20.0,
        temperature: 0.8,
        top_k: 250,
        ..Default::default()
    };

    let mut model = TextGenerationModel::new(generate_config)
        .expect("Uh oh, it's the generator broken.");
    model.set_device(Device::cuda_if_available());

    let input_context_1 = prompt;
    let output = model.generate(&[PROMPT, &input_context_1
        .to_string()
        .split_off(5)], None);

    let response: String = output.into_iter().collect();
    return response
}


pub fn ask(question: &str, context: &str) -> String {
    //    Set-up Question Answering model
    let qa_model = QuestionAnsweringModel::new(Default::default())
        .expect("Failed to initialize model.");

    //    Define input
    let question_1 = String::from(question);
    let context_1 = String::from(context);
    let qa_input_1 = QaInput {
        question: question_1,
        context: context_1,
    };

    //    Get answer
    let answers = qa_model.predict(&[qa_input_1], 1, 32);
    return format!("{:?}", answers);
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

pub fn gen(prompt: &str) -> String {
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
        min_length: 15,
        max_length: Some(100),
        do_sample: false,
        early_stopping: true,
        num_beams: 8,
        num_return_sequences: 1,
        repetition_penalty: 30.0,
        length_penalty: 4.0,
        temperature: 1.7,
        device: Device::Cpu,
        ..Default::default()
    };

    let mut model = TextGenerationModel::new(generate_config)
        .expect("This regularly blows up");
    model.set_device(Device::cuda_if_available());

    let input_context_1 = prompt;
    let mut output = model.generate(&[input_context_1], None);
    return output.into_iter().collect()
}