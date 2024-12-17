use rig::{agent::AgentBuilder, completion::CompletionModel, embeddings::EmbeddingModel};
use tracing::info;

use crate::{character::Character, knowledge::KnowledgeBase};

#[derive(Clone)]
pub struct Agent<M: CompletionModel, E: EmbeddingModel + 'static> {
    pub character: Character,
    completion_model: M,
    knowledge: KnowledgeBase<E>,
}

impl<M: CompletionModel, E: EmbeddingModel> Agent<M, E> {
    pub fn new(character: Character, completion_model: M, knowledge: KnowledgeBase<E>) -> Self {
        info!(name = character.name, "Creating new agent");

        Self {
            character,
            completion_model,
            knowledge,
        }
    }

    pub fn builder(&self) -> AgentBuilder<M> {
        // Build character context
        let character_context = format!(
            "Your name is: {}
            
            Your identity and expertise:
            Topics of expertise: {}
            
            Example messages for reference:
            {}",
            self.character.name,
            self.character.topics.join(", "),
            self.character.message_examples.join("\n")
        );

        // Build style context
        let style_context = format!(
            "Your personality and communication style:
            
            Core traits and behaviors:
            {}
            
            Communication style:
            - In chat: {}
            - In posts: {}
            
            Expression elements:
            - Common adjectives: {}
            - Expressions and reactions: {}
            
            Personal elements:
            - Key interests: {}
            - Meme-related phrases: {}",
            self.character.style.all.join("\n"),
            self.character.style.chat.join("\n"),
            self.character.style.post.join("\n"),
            self.character.style.adjectives.join(", "),
            self.character.style.expressions.join("\n"),
            self.character.style.interests.join("\n"),
            self.character.style.meme_phrases.join("\n")
        );

        let builder = AgentBuilder::new(self.completion_model.clone())
            .preamble(&self.character.preamble)
            .context(&character_context)
            .context(&style_context)
            .dynamic_context(2, self.knowledge.clone().document_index());

        builder
    }

    pub fn knowledge(&self) -> &KnowledgeBase<E> {
        &self.knowledge
    }
}