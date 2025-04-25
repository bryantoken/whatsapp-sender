use eframe::{egui, App, CreationContext};
use egui::{Button, CentralPanel, Context, ScrollArea, TextEdit, TopBottomPanel, Ui};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

mod excel_handler;
mod message_handler;
mod whatsapp_automation;

use excel_handler::ExcelHandler;
use message_handler::MessageHandler;

struct WhatsAppSenderApp {
    excel_path: String,
    message_template: String,
    delay_seconds: u32,
    status_text: String,
    progress: f32,
    excel_handler: Option<ExcelHandler>,
    message_handler: MessageHandler,
    is_sending: bool,
    sending_thread: Option<thread::JoinHandle<()>>,
    contacts_preview: String,
}

impl Default for WhatsAppSenderApp {
    fn default() -> Self {
        Self {
            excel_path: String::new(),
            message_template: String::from("Olá {nome}, tudo bem? Gostaria de conversar sobre..."),
            delay_seconds: 10,
            status_text: String::from("Pronto para iniciar."),
            progress: 0.0,
            excel_handler: None,
            message_handler: MessageHandler::new(),
            is_sending: false,
            sending_thread: None,
            contacts_preview: String::new(),
        }
    }
}

impl App for WhatsAppSenderApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Enviador de Mensagens WhatsApp");
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                self.render_excel_section(ui);
                ui.add_space(10.0);
                self.render_message_section(ui);
                ui.add_space(10.0);
                self.render_settings_section(ui);
                ui.add_space(10.0);
                self.render_status_section(ui);
            });
        });

        // Solicitar repintura contínua se estiver enviando mensagens
        if self.is_sending {
            ctx.request_repaint();
        }
    }
}

impl WhatsAppSenderApp {
    fn new(_cc: &CreationContext<'_>) -> Self {
        Self::default()
    }

    fn render_excel_section(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("Arquivo Excel");
            ui.horizontal(|ui| {
                ui.label("Caminho: ");
                ui.text_edit_singleline(&mut self.excel_path);
                if ui.button("Selecionar").clicked() {
                    // Em uma implementação completa, abriríamos um diálogo de arquivo aqui
                    // Como simplificação, apenas simulamos a seleção
                    self.status_text = "Selecione um arquivo Excel com colunas 'Nome' e 'Numero'.".to_string();
                }
            });

            if ui.button("Visualizar Contatos").clicked() {
                if !self.excel_path.is_empty() {
                    match ExcelHandler::new(&self.excel_path) {
                        Ok(handler) => {
                            self.excel_handler = Some(handler);
                            self.contacts_preview = self.excel_handler.as_ref().unwrap().get_preview(5);
                            self.status_text = format!(
                                "Arquivo carregado com sucesso. {} contatos encontrados.",
                                self.excel_handler.as_ref().unwrap().get_contact_count()
                            );
                        }
                        Err(e) => {
                            self.status_text = format!("Erro ao carregar arquivo: {}", e);
                        }
                    }
                } else {
                    self.status_text = "Selecione um arquivo Excel primeiro.".to_string();
                }
            }

            if !self.contacts_preview.is_empty() {
                ui.collapsing("Prévia dos Contatos", |ui| {
                    ui.monospace(&self.contacts_preview);
                });
            }
        });
    }

    fn render_message_section(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("Mensagem");
            ui.label("Digite sua mensagem abaixo. Use {nome} para inserir o nome do contato:");
            
            ui.horizontal(|ui| {
                if ui.button("Usar Template").clicked() {
                    // Em uma implementação completa, abriríamos um diálogo de templates
                    self.message_template = "Olá {nome}, tudo bem?\n\nEstou entrando em contato para informar sobre nossa nova promoção.\nGostaria de saber se você tem interesse em conhecer mais detalhes.\n\nAguardo seu retorno!".to_string();
                }
            });
            
            ui.text_edit_multiline(&mut self.message_template);
        });
    }

    fn render_settings_section(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("Configurações");
            ui.horizontal(|ui| {
                ui.label("Tempo entre mensagens (segundos): ");
                ui.add(egui::Slider::new(&mut self.delay_seconds, 5..=60));
            });
        });
    }

    fn render_status_section(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("Status");
            ui.label(&self.status_text);
            
            // Barra de progresso
            if self.progress > 0.0 {
                ui.label("Progresso:");
                let progress_bar = egui::ProgressBar::new(self.progress).show_percentage();
                ui.add(progress_bar);
            }
            
            ui.horizontal(|ui| {
                let button_text = if self.is_sending { "Parar Envio" } else { "Iniciar Envio" };
                let button = ui.add_enabled(!self.excel_path.is_empty() || self.is_sending, Button::new(button_text));
                
                if button.clicked() {
                    if self.is_sending {
                        self.stop_sending();
                    } else {
                        self.start_sending();
                    }
                }
            });
        });
    }

    fn start_sending(&mut self) {
        if self.excel_handler.is_none() {
            match ExcelHandler::new(&self.excel_path) {
                Ok(handler) => {
                    self.excel_handler = Some(handler);
                }
                Err(e) => {
                    self.status_text = format!("Erro ao carregar arquivo: {}", e);
                    return;
                }
            }
        }

        let excel_handler = self.excel_handler.as_ref().unwrap().clone();
        let _message_template = self.message_template.clone();
        let delay_seconds = self.delay_seconds;
        
        // Criar canais para comunicação entre threads
        let progress = Arc::new(Mutex::new(0.0));
        let status = Arc::new(Mutex::new(String::from("Iniciando envio...")));
        let is_running = Arc::new(Mutex::new(true));
        
        let progress_clone = Arc::clone(&progress);
        let status_clone = Arc::clone(&status);
        let is_running_clone = Arc::clone(&is_running);
        
        // Iniciar thread de envio
        let handle = thread::spawn(move || {
            // Simular inicialização do navegador
            *status_clone.lock().unwrap() = String::from("Iniciando o navegador...");
            thread::sleep(Duration::from_secs(2));
            
            // Simular carregamento do WhatsApp Web
            *status_clone.lock().unwrap() = String::from("Carregando WhatsApp Web...");
            thread::sleep(Duration::from_secs(2));
            
            // Simular aguardando login
            *status_clone.lock().unwrap() = String::from("Aguardando login no WhatsApp Web...\nPor favor, escaneie o código QR.");
            thread::sleep(Duration::from_secs(3));
            
            // Simular login bem-sucedido
            *status_clone.lock().unwrap() = String::from("Login realizado com sucesso!");
            thread::sleep(Duration::from_secs(1));
            
            // Em uma implementação real, usaríamos o WhatsAppAutomation aqui
            let contacts = excel_handler.get_contacts();
            let total = contacts.len();
            
            *status_clone.lock().unwrap() = format!("Enviando mensagens para {} contatos...", total);
            
            // Simular envio de mensagens
            for (i, contact) in contacts.iter().enumerate() {
                if !*is_running_clone.lock().unwrap() {
                    break;
                }
                
                let nome = &contact.nome;
                let _numero = &contact.numero;
                
                *status_clone.lock().unwrap() = format!("Enviando para {} ({}/{})", nome, i + 1, total);
                
                // Simular envio (em uma implementação real, usaríamos WhatsAppAutomation)
                thread::sleep(Duration::from_secs(delay_seconds as u64));
                
                // Atualizar progresso
                *progress_clone.lock().unwrap() = (i as f32 + 1.0) / total as f32;
            }
            
            if *is_running_clone.lock().unwrap() {
                *status_clone.lock().unwrap() = String::from("Envio concluído com sucesso!");
            } else {
                *status_clone.lock().unwrap() = String::from("Envio interrompido pelo usuário.");
            }
        });
        
        self.sending_thread = Some(handle);
        self.is_sending = true;
        
        // Em uma implementação real, usaríamos um canal para comunicação entre threads
        // Como simplificação, apenas atualizamos diretamente
        self.progress = 0.0;
        self.status_text = String::from("Iniciando envio...");
        
        // Iniciar thread para atualizar UI
        let _progress_app = Arc::clone(&progress);
        let _status_app = Arc::clone(&status);
        
        thread::spawn(move || {
            while *is_running.lock().unwrap() {
                thread::sleep(Duration::from_millis(100));
                // Em uma implementação real, atualizaríamos a UI aqui
            }
        });
    }

    fn stop_sending(&mut self) {
        self.is_sending = false;
        self.status_text = String::from("Interrompendo envio...");
        // Em uma implementação real, sinalizaríamos para a thread parar
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Enviador de Mensagens WhatsApp",
        options,
        Box::new(|cc| Box::new(WhatsAppSenderApp::new(cc))),
    )
}
