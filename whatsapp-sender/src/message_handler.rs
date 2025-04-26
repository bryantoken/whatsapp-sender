use regex::Regex;
use std::collections::HashMap;

#[derive(Default)]
pub struct MessageHandler {
    template: String,
    placeholders: Vec<String>,
}

impl MessageHandler {
    pub fn new() -> Self {
        Self {
            template: String::new(),
            placeholders: Vec::new(),
        }
    }
    
    pub fn set_template(&mut self, template: &str) -> &[String] {
        self.template = template.to_string();
        
        // Encontrar todos os placeholders no formato {nome_placeholder}
        let re = Regex::new(r"\{([^}]+)\}").unwrap();
        self.placeholders = re
            .captures_iter(template)
            .map(|cap| cap[1].to_string())
            .collect();
        
        &self.placeholders
    }
    
    pub fn personalize_message(&self, contact_data: &HashMap<String, String>) -> String {
        if self.template.is_empty() {
            return String::new();
        }
        
        let mut message = self.template.clone();
        let empty = String::new();
        
        // Substituir cada placeholder pelos dados do contato
        for placeholder in &self.placeholders {
            let placeholder_lower = placeholder.to_lowercase();
            
            // Mapear 'nome' para 'Nome' explicitamente
            let value = if placeholder_lower == "nome" {
                contact_data.get("Nome")
                    .or_else(|| contact_data.get("nome"))
                    .unwrap_or(&empty)
            } else {
                contact_data.get(placeholder)
                    .unwrap_or(&empty)
            };
            
            message = message.replace(&format!("{{{}}}", placeholder), value);
        }
        
        message
    }
    
    pub fn get_default_template() -> String {
        String::from(
            "Olá {nome}, tudo bem?\n\n\
            Estou entrando em contato para informar sobre nossa nova promoção.\n\
            Gostaria de saber se você tem interesse em conhecer mais detalhes.\n\n\
            Aguardo seu retorno!"
        )
    }
    
    pub fn get_template_examples() -> Vec<(String, String)> {
        vec![
            (
                String::from("Saudação Simples"),
                String::from("Olá {nome}, tudo bem? Espero que esteja tendo um ótimo dia!")
            ),
            (
                String::from("Informação de Conta"),
                String::from("A conta sua, {nome}, está pronta para ser acessada. Entre em contato para mais informações.")
            ),
            (
                String::from("Convite para Evento"),
                String::from("Olá {nome}! Gostaríamos de convidá-lo(a) para nosso evento que acontecerá no próximo sábado.")
            ),
            (
                String::from("Confirmação de Agendamento"),
                String::from("Prezado(a) {nome}, confirmamos seu agendamento para o dia 10/05/2025 às 14h.")
            ),
            (
                String::from("Promoção Personalizada"),
                String::from("Oi {nome}! Preparamos uma oferta especial para você. Responda esta mensagem para saber mais.")
            ),
        ]
    }
    
    pub fn add_signature(&self, message: &str, signature: &str) -> String {
        if signature.is_empty() {
            return message.to_string();
        }
        
        // Adicionar duas linhas em branco antes da assinatura
        format!("{}\n\n{}", message, signature)
    }
    
    pub fn format_message(&self, message: &str) -> String {
        // Remover espaços em branco extras
        let re_spaces = Regex::new(r"\s+").unwrap();
        let message = re_spaces.replace_all(message, " ");
        
        // Garantir que parágrafos estejam separados por uma linha em branco
        let re_paragraphs = Regex::new(r"(\n)(\S)").unwrap();
        let message = re_paragraphs.replace_all(&message, "$1\n$2");
        
        // Remover linhas em branco consecutivas (mais de duas)
        let re_blank_lines = Regex::new(r"\n{3,}").unwrap();
        let message = re_blank_lines.replace_all(&message, "\n\n");
        
        message.trim().to_string()
    }
    
    pub fn preview_message(&self, contact_data: &HashMap<String, String>) -> String {
        let personalized = self.personalize_message(contact_data);
        
        // Adicionar informações de prévia
        format!(
            "PRÉVIA DA MENSAGEM\n\
            ====================\n\
            {}\n\
            ====================",
            personalized
        )
    }
}
