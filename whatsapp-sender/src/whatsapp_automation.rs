use anyhow::{Context, Result};
use std::time::Duration;
use thirtyfour::{By, DesiredCapabilities, WebDriver};
use tokio::time::sleep;

pub struct WhatsAppAutomation {
    driver: Option<WebDriver>,
    is_logged_in: bool,
}

impl WhatsAppAutomation {
    pub fn new() -> Self {
        Self {
            driver: None,
            is_logged_in: false,
        }
    }
    
    pub async fn initialize(&mut self) -> Result<()> {
        // Configurar o Firefox WebDriver
        let caps = DesiredCapabilities::firefox();
        
        // Inicializar o WebDriver
        let driver = WebDriver::new("http://localhost:4444", caps).await
            .context("Falha ao inicializar o WebDriver. Verifique se o geckodriver está instalado e em execução.")?;
        
        self.driver = Some(driver);
        Ok(())
    }
    
    pub async fn load_whatsapp_web(&mut self) -> Result<()> {
        let driver = self.driver.as_ref()
            .ok_or_else(|| anyhow::anyhow!("WebDriver não inicializado"))?;
        
        // Carregar o WhatsApp Web
        driver.goto("https://web.whatsapp.com/").await
            .context("Falha ao carregar o WhatsApp Web")?;
        
        Ok(())
    }
    
    pub async fn wait_for_login(&mut self, _timeout_seconds: u64) -> Result<()> {
        let driver = self.driver.as_ref()
            .ok_or_else(|| anyhow::anyhow!("WebDriver não inicializado"))?;
        
        // Esperar até que a página principal do WhatsApp seja carregada
        // Isso é indicado pela presença do campo de mensagem
        driver.find(By::XPath("//div[@contenteditable='true'][@data-tab='3']"))
            .await
            .context("Tempo esgotado aguardando o login")?;
        
        self.is_logged_in = true;
        Ok(())
    }
    
    pub async fn send_message(&self, numero: &str, mensagem: &str) -> Result<()> {
        if !self.is_logged_in {
            return Err(anyhow::anyhow!("Não está logado no WhatsApp Web"));
        }
        
        let driver = self.driver.as_ref()
            .ok_or_else(|| anyhow::anyhow!("WebDriver não inicializado"))?;
        
        // Codificar a mensagem para URL
        let mensagem_encoded = mensagem.replace(" ", "%20");
        
        // Construir a URL do WhatsApp
        let url = format!("https://web.whatsapp.com/send?phone={}&text={}", numero, mensagem_encoded);
        
        // Navegar para a URL
        driver.goto(&url).await
            .context("Falha ao navegar para a página de conversa")?;
        
        // Esperar até que a página de conversa seja carregada
        let send_button = driver.find(By::XPath("//button[@aria-label='Enviar']"))
            .await
            .context("Tempo esgotado aguardando a página de conversa")?;
        
        // Pequena pausa para garantir que tudo carregou
        sleep(Duration::from_secs(2)).await;
        
        // Clicar no botão de enviar
        send_button.click().await
            .context("Falha ao clicar no botão de enviar")?;
        
        // Esperar o envio
        sleep(Duration::from_secs(2)).await;
        
        Ok(())
    }
    
    pub async fn close(&mut self) -> Result<()> {
        if let Some(driver) = self.driver.take() {
            driver.quit().await
                .context("Falha ao fechar o WebDriver")?;
        }
        
        self.is_logged_in = false;
        Ok(())
    }
    
    // Função auxiliar para formatar números de telefone
    pub fn format_phone_number(numero: &str) -> String {
        // Remover caracteres não numéricos
        let numero = numero.chars()
            .filter(|c| c.is_digit(10) || *c == '+')
            .collect::<String>();
        
        // Garantir que o número tenha o formato internacional
        if !numero.starts_with('+') {
            // Assumir Brasil como padrão se não tiver código de país
            if numero.len() <= 11 {
                return format!("+55{}", numero);
            } else {
                return format!("+{}", numero);
            }
        }
        
        numero
    }
    
    // Função para verificar se o WhatsApp Web está disponível
    pub async fn check_whatsapp_web_availability() -> Result<bool> {
        // Em uma implementação real, verificaríamos a disponibilidade do WhatsApp Web
        // Como simplificação, apenas retornamos true
        Ok(true)
    }
}
