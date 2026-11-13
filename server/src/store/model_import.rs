//! Imports model definitions from exported JSON files.
use std::collections::HashSet;

use crate::{
    model::{model_hash, normalize_model_input, ModelConfigInput},
    Error, Result,
};

use super::Store;

pub struct ModelImportPlan {
    pub models: Vec<ModelImportEntry>,
}

pub struct ModelImportEntry {
    pub model_hash: String,
    pub input: ModelConfigInput,
    pub existing: bool,
}

pub struct ModelImportOutcome {
    pub imported: usize,
    pub skipped: usize,
    pub total: usize,
}

impl Store {
    /// 逐条规范化导入配置并标记是否已存在;文件内重复的配置只保留第一条。
    pub async fn preview_model_import(
        &self,
        inputs: &[ModelConfigInput],
    ) -> Result<ModelImportPlan> {
        if inputs.is_empty() {
            return Err(Error::Config("the import file contains no models".into()));
        }
        let existing = self
            .models()
            .await?
            .into_iter()
            .map(|model| model.model_hash)
            .collect::<HashSet<_>>();
        let mut seen = HashSet::with_capacity(inputs.len());
        let mut models = Vec::with_capacity(inputs.len());
        for input in inputs {
            let input = normalize_model_input(input)?;
            let hash = model_hash(&input)?;
            if seen.insert(hash.clone()) {
                models.push(ModelImportEntry {
                    existing: existing.contains(&hash),
                    model_hash: hash,
                    input,
                });
            }
        }
        Ok(ModelImportPlan { models })
    }

    /// 把导入的模型追加到现有列表末尾;已存在的模型原样保留。
    pub async fn import_models(&self, inputs: &[ModelConfigInput]) -> Result<ModelImportOutcome> {
        let plan = self.preview_model_import(inputs).await?;
        let total = plan.models.len();
        let next_order =
            i64::try_from(self.models().await?.len()).expect("model order fits in i64") + 1;
        let missing = plan
            .models
            .into_iter()
            .filter(|model| !model.existing)
            .enumerate()
            .map(|(index, model)| ModelConfigInput {
                sort_order: next_order + i64::try_from(index).expect("model order fits in i64"),
                ..model.input
            })
            .collect::<Vec<_>>();
        let imported = self.create_models_if_missing(&missing).await?;
        Ok(ModelImportOutcome {
            imported,
            skipped: total - imported,
            total,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model_input(display_name: &str, model_id: &str) -> ModelConfigInput {
        ModelConfigInput {
            sort_order: 0,
            display_name: display_name.into(),
            group_name: None,
            model_type: crate::model::ModelType::OpenAi,
            base_url: "https://example.com/v1/chat/completions".into(),
            use_full_url: true,
            api_key: "test-key".into(),
            tooltip_data: display_name.into(),
            model_id: model_id.into(),
            reasoning_effort: None,
            openai_endpoint: crate::model::OPENAI_CHAT_ENDPOINT.into(),
            openai_extra_params_enabled: false,
            openai_extra_params: serde_json::json!({}),
            custom_headers_enabled: false,
            custom_headers: serde_json::json!({}),
            anthropic_extra_params_enabled: false,
            anthropic_extra_params: serde_json::json!({}),
            context_window_tokens: None,
            max_completion_tokens: None,
            anthropic_max_tokens: None,
            anthropic_thinking_effort: None,
            thinking_budget_tokens: None,
        }
    }

    /// 导入保留已存在模型,把新模型追加到末尾,并且文件内重复项只保留一条。
    #[tokio::test]
    async fn import_appends_new_models_and_skips_existing() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::connect(&format!(
            "sqlite://{}",
            directory.path().join("test.db").display()
        ))
        .await
        .unwrap();
        store
            .create_model(&model_input("First", "first-model"))
            .await
            .unwrap();

        let plan = store
            .preview_model_import(&[
                model_input("First", "first-model"),
                model_input("Second", "second-model"),
                model_input("Second", "second-model"),
            ])
            .await
            .unwrap();
        assert_eq!(plan.models.len(), 2);
        assert!(plan.models[0].existing);
        assert!(!plan.models[1].existing);

        let outcome = store
            .import_models(&[
                model_input("First", "first-model"),
                model_input("Second", "second-model"),
            ])
            .await
            .unwrap();
        assert_eq!(outcome.imported, 1);
        assert_eq!(outcome.skipped, 1);
        assert_eq!(outcome.total, 2);

        let models = store.models().await.unwrap();
        assert_eq!(models.len(), 2);
        assert_eq!(models[1].display_name, "Second");
        assert_eq!(models[1].sort_order, 2);
    }
}
