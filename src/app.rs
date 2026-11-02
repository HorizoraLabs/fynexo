use std::error::Error;
use tao::{
    event::{Event, StartCause, WindowEvent}, 
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy}, 
    window::WindowBuilder,
};
use wry::WebViewBuilder;
use crate::hud;

#[cfg(target_os = "windows")]
use wry::WebViewBuilderExtWindows;

pub enum UserEvent {
    Navigate(String),
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let event_proxy: EventLoopProxy<UserEvent> = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("Fynexo")
        .with_inner_size(tao::dpi::LogicalSize::new(1024.0, 768.0))
        .build(&event_loop)?;


    let mut builder = WebViewBuilder::new();

    #[cfg(target_os = "windows")]
    {
        builder = builder.with_additional_browser_args(
            "--renderer-process-limit=1 \
            --js-flags=\"--max-old-space-size=96 --optimize-for0size\" \
            --disable-gpu-shader-disk-cache \
            --disable-features=Prerender2,TopicsAPI \
            --disk_cache-size=33554432 \
            ",
        );
    }

    let webview = builder
        .with_devtools(true)
        .with_initialization_script(hud::SCRIPT)
        .with_initialization_script(hud::INIT)
        .with_ipc_handler(move |req| {
            let body = req.body();
            println!("[IPC Request]: {}", body);
            
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
                if val["type"] == "NAVIGATE" {
                    if let Some(target_url) = val["url"].as_str() {
                        let formatted_url = if !target_url.starts_with("http://") && !target_url.starts_with("https://") {
                            format!("https://{}", target_url)
                        } else {
                            target_url.to_string()
                        };
                        let _ = event_proxy.clone().send_event(UserEvent::Navigate(formatted_url));
                    }
                }
            }
        })
        .with_url("https://www.google.com")
        .build(&window)?;

     // Run the Window Event Lifecycle
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => (),
            Event::UserEvent(UserEvent::Navigate(url)) => {
                println!("[Navigation Request]: {}", url);
                let _ = webview.load_url(&url);
            }
            Event::WindowEvent { 
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });

    #[allow(unreachable_code)]
    Ok(())
}