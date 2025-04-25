use anyhow::{Context, Result};
use calamine::{open_workbook, Reader, Xlsx, DataType};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub nome: String,
    pub numero: String,
    // Campos adicionais opcionais
    pub email: Option<String>,
    pub empresa: Option<String>,
}

#[derive(Clone)]
pub struct ExcelHandler {
    file_path: String,
    contacts: Vec<Contact>,
}

impl ExcelHandler {
    pub fn new(file_path: &str) -> Result<Self> {
        let path = Path::new(file_path);
        
        // Verificar se o arquivo existe
        if !path.exists() {
            return Err(anyhow::anyhow!("O arquivo {} não existe", file_path));
        }
        
        // Verificar se é um arquivo Excel
        if !file_path.ends_with(".xlsx") && !file_path.ends_with(".xls") {
            return Err(anyhow::anyhow!("O arquivo {} não é um arquivo Excel válido", file_path));
        }
        
        // Abrir o arquivo Excel
        let mut workbook: Xlsx<_> = open_workbook(file_path)
            .with_context(|| format!("Erro ao abrir o arquivo Excel: {}", file_path))?;
        
        // Obter a primeira planilha
        let sheet_name = workbook.sheet_names().get(0)
            .ok_or_else(|| anyhow::anyhow!("O arquivo Excel não contém planilhas"))?
            .clone();
        
        let sheet = workbook.worksheet_range(&sheet_name)
            .with_context(|| "Erro ao acessar a planilha")?;
        
        // Verificar se a planilha está vazia
        if sheet.is_empty() {
            return Err(anyhow::anyhow!("A planilha está vazia"));
        }
        
        // Encontrar os índices das colunas necessárias
        let mut nome_idx = None;
        let mut numero_idx = None;
        let mut email_idx = None;
        let mut empresa_idx = None;
        
        if let Some(header_row) = sheet.rows().next() {
            for (i, cell) in header_row.iter().enumerate() {
                if let Some(value) = cell.get_string() {
                    match value.to_lowercase().as_str() {
                        "nome" => nome_idx = Some(i),
                        "numero" => numero_idx = Some(i),
                        "telefone" => numero_idx = Some(i),
                        "email" => email_idx = Some(i),
                        "empresa" => empresa_idx = Some(i),
                        _ => {}
                    }
                }
            }
        }
        
        // Verificar se as colunas obrigatórias existem
        let nome_idx = nome_idx.ok_or_else(|| anyhow::anyhow!("Coluna 'Nome' não encontrada"))?;
        let numero_idx = numero_idx.ok_or_else(|| anyhow::anyhow!("Coluna 'Numero' ou 'Telefone' não encontrada"))?;
        
        // Extrair os contatos
        let mut contacts = Vec::new();
        
        for row in sheet.rows().skip(1) {
            if row.len() <= nome_idx || row.len() <= numero_idx {
                continue;
            }
            
            let nome = row[nome_idx].get_string().unwrap_or_default().to_string();
            let numero = row[numero_idx].get_string().unwrap_or_default().to_string();
            
            // Pular linhas com dados incompletos
            if nome.is_empty() || numero.is_empty() {
                continue;
            }
            
            // Extrair campos opcionais
            let email = email_idx.and_then(|idx| {
                if idx < row.len() {
                    row[idx].get_string().map(|s| s.to_string())
                } else {
                    None
                }
            });
            
            let empresa = empresa_idx.and_then(|idx| {
                if idx < row.len() {
                    row[idx].get_string().map(|s| s.to_string())
                } else {
                    None
                }
            });
            
            contacts.push(Contact {
                nome,
                numero,
                email,
                empresa,
            });
        }
        
        // Verificar se há contatos
        if contacts.is_empty() {
            return Err(anyhow::anyhow!("Nenhum contato válido encontrado no arquivo"));
        }
        
        Ok(Self {
            file_path: file_path.to_string(),
            contacts,
        })
    }
    
    pub fn get_contacts(&self) -> &[Contact] {
        &self.contacts
    }
    
    pub fn get_contact_count(&self) -> usize {
        self.contacts.len()
    }
    
    pub fn get_preview(&self, max_rows: usize) -> String {
        let mut preview = String::new();
        
        preview.push_str(&format!("Prévia dos contatos (primeiros {} de {}):\n\n", 
            std::cmp::min(max_rows, self.contacts.len()), self.contacts.len()));
        
        // Adicionar cabeçalhos
        preview.push_str("Nome | Numero | Email | Empresa\n");
        preview.push_str("-----|--------|-------|--------\n");
        
        // Adicionar linhas
        for (i, contact) in self.contacts.iter().enumerate() {
            if i >= max_rows {
                break;
            }
            
            let email = contact.email.as_deref().unwrap_or("-");
            let empresa = contact.empresa.as_deref().unwrap_or("-");
            
            preview.push_str(&format!("{} | {} | {} | {}\n", 
                contact.nome, contact.numero, email, empresa));
        }
        
        preview
    }
    
    pub fn save_results(&self, _results: &[(usize, String)], output_path: Option<&str>) -> Result<String> {
        // Em uma implementação completa, salvaríamos os resultados em um novo arquivo Excel
        // Como simplificação, apenas retornamos o caminho onde seria salvo
        
        let output_path = match output_path {
            Some(path) => path.to_string(),
            None => {
                let path = Path::new(&self.file_path);
                let stem = path.file_stem().unwrap().to_string_lossy();
                let ext = path.extension().unwrap().to_string_lossy();
                format!("{}_resultados.{}", stem, ext)
            }
        };
        
        // Em uma implementação real, usaríamos rust_xlsxwriter para criar o arquivo
        
        Ok(output_path)
    }
}
